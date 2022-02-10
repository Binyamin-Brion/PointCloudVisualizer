use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use glfw::OpenGlProfileHint;
use nalgebra_glm::{TMat4, TVec3, vec3};
use crate::args_parser;
use crate::args_parser::Args;
use crate::rendering::scene_renderer::{SceneRenderer, ModelId};
use crate::rendering::camera::{Camera, CameraType, PerspectiveParam};
use crate::ipc_logic::ipc_content_logic::ClusterInformation;
use crate::ipc_logic::ipc_receiver::{IPCContributor, SendContents};
use crate::helper_logic::point_cloud_analyzer::InitialCloudAnalyzer;
use crate::rendering::text_rendering::TextRendering;
use crate::rendering::view_fbo::ViewFBO;
use crate::view_logic::view_selection::ViewSelection;
use crate::window::RenderWindow;

/// Holds all of the variables used in the program
pub struct ProgramVariables
{
    pub args: Args,
    pub point_analyzer: InitialCloudAnalyzer,
    pub point_cloud_data: PointCloudData,
    pub point_cloud_update: PointCloudUpdate,
    pub render_data: RenderData,
    have_centred_views: bool,
}

/// Holds all of the variables containing information
/// regarding the point cloud
pub struct PointCloudData
{
    pub position: Option<TVec3<f32>>,
    pub time_since_update: Instant,
    pub pause_updating: bool,
    pub cluster_result_text: String,
    pub num_points_cloud: usize,
    pub cluster_information: ClusterInformation,
}

/// Holds all of the variables required for updating
/// the point cloud
pub struct PointCloudUpdate
{
    pub current_content_file: String,
    pub cluster_for_most_recent: bool,
    pub sender: SyncSender<Result<SendContents, String>>,
    pub receiver: Receiver<Result<SendContents, String>>,
    quit_ipc_thread: Arc<Mutex<bool>>
}

/// Holds all of the required variables for the rendering done
/// in the program
pub struct RenderData
{
    pub buffer_groups: SceneRenderer,
    pub cube_model_id: ModelId,
    pub render_window: RenderWindow,
    pub camera: Camera,
    pub translation_matrix: TMat4<f32>,
    pub view_selection: ViewSelection,
    pub view_fbos: ViewFBO,
    pub text_renderer: TextRendering,
    pub cloud_translation: TVec3<f32>,
    pub add_lidar_pos: bool,
    reflect_vertically: i32,
}

impl ProgramVariables
{
    /// Creates the all of the program variables required for the
    /// program to run
    pub fn new() -> ProgramVariables
    {
        let args = args_parser::Args::parse_args();
        let point_analyzer = InitialCloudAnalyzer::new(&args.initial_data_model, args.display_lidar_pos);

        let mut program_variables = ProgramVariables
        {
            render_data: RenderData::new(&point_analyzer),
            point_cloud_data: PointCloudData::new(&args, &point_analyzer),
            point_cloud_update: PointCloudUpdate::new(&args),
            args,
            point_analyzer,
            have_centred_views: false
        };

        // If the point cloud is being updated, then the the cameras will be
        // centred after the first update of the point cloud. This is because logically
        // an initial point cloud will not be provided if the point clouds
        // are going to be updated
        if !program_variables.args.using_file_ipc()
        {
            program_variables.centre_views(program_variables.args.display_lidar_pos);
        }

        program_variables
    }

    /// Centres the camera views based off of the location of the point cloud
    pub fn centre_views(&mut self, displaying_lidar_pos: bool)
    {
        if !self.have_centred_views
        {
            self.point_analyzer = InitialCloudAnalyzer::new(&Some(self.point_cloud_update.current_content_file.clone()), displaying_lidar_pos);

            let mut right_pos = self.point_analyzer.get_centre();
            right_pos -= self.render_data.view_fbos.get_right_fbo().get_camera().get_direction() * self.point_analyzer.get_max_length();

            let mut top_pos = self.point_analyzer.get_centre();
            top_pos -= self.render_data.view_fbos.get_top_fbo().get_camera().get_direction() * self.point_analyzer.get_max_length();

            // The values of "3" were provided as based off of different point
            // clouds provided, it provided a good offset for the cameras. Worst case
            // the user moves the camera to a desired location

            let mut sun_pos = self.point_analyzer.get_centre();
            sun_pos -= self.render_data.view_fbos.get_sun_fbo().get_sun_direction() * 3.0;

            let mut main_camera_pos = self.point_analyzer.get_centre();
            main_camera_pos -= self.render_data.camera.get_direction() * 3.0;

            self.render_data.camera.set_camera_pos(main_camera_pos);
            self.render_data.view_fbos.hard_set_light_pos(sun_pos, self.point_analyzer.get_centre());
            self.render_data.view_fbos.hard_set_right_view_pos(right_pos);
            self.render_data.view_fbos.hard_set_top_view_pos(top_pos);
        }

        self.have_centred_views = true;
    }
}

impl RenderData
{
    /// Creates the variables required to perform rendering operations in the scene
    ///
    /// `point_analyzer` - information about the inital point cloud (if none is provided,
    ///                     the InitialCloudAnalyzer will take that into account
    fn new(point_analyzer: &InitialCloudAnalyzer) -> RenderData
    {
        let render_window = create_window((1280, 720), "Point Cloud Visualizer".to_string());
        let (buffer_groups, cube_model_id) = SceneRenderer::setup_scene_renderer(point_analyzer);

        RenderData
        {
            buffer_groups,
            cube_model_id,
            text_renderer: TextRendering::new(render_window.get_window_dimensions()),
            camera: setup_default_camera(&render_window),
            view_fbos: ViewFBO::new(&render_window),
            render_window,
            translation_matrix: setup_translation_matrix(),
            view_selection: ViewSelection::new(),
            cloud_translation: vec3(0.0, 0.0, 0.0),
            reflect_vertically: 1,
            add_lidar_pos: false
        }
    }

    pub fn get_reflect_vertically(&self) -> i32
    {
        self.reflect_vertically
    }

    pub fn reflect_y_axis(&mut self)
    {
        self.reflect_vertically *= -1;
    }
}

impl PointCloudUpdate
{
    /// Creates the variables required to update the point cloud
    ///
    /// `args` - the arguments passed to the program upon launching it
    fn new(args: &Args) -> PointCloudUpdate
    {
        // The initial file is the one containing the initial point cloud
        // or the first file used for updating the point cloud
        let current_content_file = if let Some(ref i) = args.initial_data_model
        {
            i.clone()
        }
        else
        {
            args.ipc_files[0].data_file_names.clone()
        };

        let (sender, receiver) = sync_channel(1);
        let quit_ipc_thread = Arc::new(Mutex::new(false));

        if args.using_file_ipc()
        {
            launch_ipc_contributor(IPCContributor::new(args.ipc_files.clone(), sender.clone(), args.sleep_duration_ms), quit_ipc_thread.clone());
        }

        PointCloudUpdate
        {
            current_content_file,
            cluster_for_most_recent: false,
            sender,
            receiver,
            quit_ipc_thread,
        }
    }

    /// Tell the cluster thread to quit
    pub fn notify_cluster_thread_to_quit(&mut self)
    {
        match self.quit_ipc_thread.lock()
        {
            Ok(mut i) => *i = true,
            Err(err) => panic!("Failed to notify cluster thread to quit: {}", err)
        }
    }
}

impl PointCloudData
{
    /// Creates analytics about the initial point cloud
    ///
    /// `args` - the arguments passed into the program when launching it
    /// `point_analyzer` - information about the initial point cloud (if none is provided,
    ///                     the InitialCloudAnalyzer will take that into account
    fn new(args: &Args, point_analyzer: &InitialCloudAnalyzer) -> PointCloudData
    {
        let cluster_information = ClusterInformation
        {
            output_file: "clusterDetectionResult.txt".to_string(),
            epsilon: 0.05,
            min_num_points: 20
        };

        PointCloudData
        {
            time_since_update: Instant::now(),
            pause_updating: false || args.initial_data_model.is_some(),
            cluster_result_text: "Cluster program status: No Error".to_string(),
            num_points_cloud: point_analyzer.get_initial_points().len(),
            cluster_information,
            position: point_analyzer.get_initial_lidar_pos()
        }
    }
}

/// Creates a window of the given size and title
///
/// `window_size` - the size the window should have
/// `window_title` - the title the created window should have
pub fn create_window(window_size: (u32, u32), window_tile: String) -> RenderWindow
{
    let window_hints = if cfg!(debug_assertions)
    {
        vec!
        [
            // Only have debug mode if the program as a whole is compiled in debug mode
            glfw::WindowHint::OpenGlDebugContext(true),
            glfw::WindowHint::ContextVersion(4, 5),
            glfw::WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
        ]
    }
    else
    {
        vec!
        [
            glfw::WindowHint::ContextVersion(4, 5),
            glfw::WindowHint::OpenGlProfile(OpenGlProfileHint::Core)
        ]
    };

    let render_window = RenderWindow::new
        (
            window_size,
            window_tile,
            window_hints,
        );

    // These are known to be needed later in the program
    unsafe
        {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::STENCIL_TEST);
        }

    render_window
}

/// Creates a default camera to use for rendering. This camera is later automatically
/// updated to be centred around the point cloud
///
/// `render_window` - the window being used for rendering
pub fn setup_default_camera(render_window: &RenderWindow) -> Camera
{
    Camera::new(CameraType::Perspective(PerspectiveParam
    {
        window_dimensions: render_window.get_window_dimensions(),
        near_plane: 0.1,
        far_plane: 100.0,
        position: vec3(0.0, 0.0, 0.0),
        direction: vec3(1.0, 0.0, 0.0),
        up: vec3(0.0, 1.0, 0.0),
    }))
}

/// Creates the matrix used
pub fn setup_translation_matrix() -> TMat4<f32>
{
    let mut translation_matrix = nalgebra_glm::identity();
    translation_matrix = nalgebra_glm::rotate(&translation_matrix, 90.0_f32.to_radians(), &vec3(0.0, 1.0, 0.0));
    translation_matrix = nalgebra_glm::translate(&translation_matrix, &vec3(0.0, 1.0, 0.0));
    translation_matrix
}

/// Launches the thread that checks for updates to the point cloud
///
/// `ipc_contributor` - variable holding required information for IPC communication
/// `quit_thread` - the variable holding the status of whether to quit the thread or not
pub fn launch_ipc_contributor(mut ipc_contributor: IPCContributor, quit_thread: Arc<Mutex<bool>>)
{
    thread::spawn(move ||
        {
            loop
            {
                ipc_contributor.read_rendering_data(&quit_thread);

                match quit_thread.lock()
                {
                    Ok(i) => if *i { break; },
                    Err(err) => panic!("Failed to check if cluster thread should quit: {}", err)
                }
            }
        });
}