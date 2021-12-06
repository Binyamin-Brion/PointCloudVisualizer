use std::time::Instant;
use glfw::{Action, Key, MouseButton};
use nalgebra_glm::vec2;
use crate::helper_logic::initialization_functions::RenderData;
use crate::rendering::scene_renderer::{SceneRenderer, ModelId, UploadInformation};
use crate::rendering::camera::Camera;
use crate::ipc_logic::ipc_content_logic::{ClusterInformation, IPCProcessingArgs, IPCUpdateResult, launch_cluster_program, process_ipc_content, read_cluster_output_file};
use crate::rendering::text_rendering::TextRendering;
use crate::rendering::view_fbo::ViewFBO;
use crate::view_logic::view_selection::ViewSelection;
use crate::window::RenderWindow;

/// Required parameters to write program information
/// to the window
pub struct TextWriteParam<'a>
{
    pub text_renderer: &'a mut TextRendering,
    pub view_fbos: &'a ViewFBO,
    pub camera: &'a mut Camera,
    pub time_update: &'a Instant,
    pub render_window: &'a RenderWindow,
    pub num_points: usize,
    pub cluster_result_text: &'a str,
    pub epsilon: f32,
    pub min_num_points: u32,
}

/// Required parameters to process a new update
/// from the IPC mechanism
pub struct HandleIPCUpdate<'a>
{
    pub ipc_args: IPCProcessingArgs<'a>,
    pub num_cloud_points: &'a mut usize,
    pub time_since_update: &'a mut Instant,
    pub cluster_result_text: &'a mut String,
    pub current_content_file: &'a mut String
}

/// Required parameters to update point cloud
pub struct HandleClusterUpdate<'a>
{
    pub buffer_groups: &'a mut SceneRenderer,
    pub buffer_update_content: &'a ClusterInformation,
    pub cube_model_id: ModelId,
    pub cluster_result_text: &'a mut String,
    pub current_content_file: &'a mut String
}

/// Checks if any of the views of the scene have been selected
///
/// `view_selection` - struct containing all of the scene views
/// `fbos` - struct containing the scene view's FBOs
/// `camera` - the main scene camera
/// `render_window` - the window being rendered to
pub fn check_for_view_selection(view_selection: &mut ViewSelection, fbos: &mut ViewFBO, camera: &mut Camera, render_window: &RenderWindow)
{
    if render_window.get_cursor_button_history().iter().find(|x| **x == (MouseButton::Button1, Action::Press)).is_some()
    {
        view_selection.update_view_selection(render_window);

        // Depending on the state of the program, the movement of a camera
        // can still occur even after a different view is selected. This
        // fixes that. These are also placed here as left-clicking should
        // terminate existing camera movement, regardless of where left-click is
        fbos.reset_movement_key_status();
        camera.clear_movement_key();
    }
}

/// Updates the position of the camera based off of the input of the user
///
/// `view_selection` - struct that handles the state of what view is selected
/// `fbos` - struct containing the scene view's FBOs
/// `camera` - the main scene camea
/// `render_window` - the window being rendered to
pub fn update_camera_movement(view_selection: &mut ViewSelection, fbos: &mut ViewFBO, camera: &mut Camera, render_window: &RenderWindow)
{
    if view_selection.is_any_view_selected()
    {
        fbos.update_camera_movement(view_selection, render_window);
    }
    else
    {
        Camera::update_camera_movement(&render_window, camera);
        Camera::update_camera_rotation(&render_window, camera);
    }
}

pub fn reflect_point_cloud(render_variables: &mut RenderData)
{
    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F7, Action::Press)).is_some()
    {
        render_variables.reflect_y_axis();
    }
}

/// Handles changes to parameters passed into the cluster detection algorithm
pub fn update_cluster_information(cluster_information: &mut ClusterInformation, cluster_for_most_recent: &mut bool, render_window: &RenderWindow)
{
    if render_window.get_key_input().iter().find(|x| **x == (Key::Z, Action::Press)).is_some() ||
        render_window.get_key_input().iter().find(|x| **x == (Key::Z, Action::Repeat)).is_some()
    {
        cluster_information.epsilon = (cluster_information.epsilon - 0.05).max(0.0);
        *cluster_for_most_recent = false;
    }

    if render_window.get_key_input().iter().find(|x| **x == (Key::X, Action::Press)).is_some() ||
        render_window.get_key_input().iter().find(|x| **x == (Key::X, Action::Repeat)).is_some()
    {
        cluster_information.epsilon += 0.05;
        *cluster_for_most_recent = false;
    }

    if render_window.get_key_input().iter().find(|x| **x == (Key::V, Action::Press)).is_some() ||
        render_window.get_key_input().iter().find(|x| **x == (Key::V, Action::Repeat)).is_some()
    {
        cluster_information.min_num_points = (cluster_information.min_num_points - 1).max(1);
        *cluster_for_most_recent = false;
    }

    if render_window.get_key_input().iter().find(|x| **x == (Key::B, Action::Press)).is_some() ||
        render_window.get_key_input().iter().find(|x| **x == (Key::B, Action::Repeat)).is_some()
    {
        cluster_information.min_num_points += 1;
        *cluster_for_most_recent = false;
    }
}

/// Updates the point cloud based off of the update provided the IPC mechanism
///
/// `args` - struct containing the variables required to handle an IPC update
pub fn update_point_cloud(args: HandleIPCUpdate)
{
    // This reads the update of the IPC and then launches the cluster program (at this point it is the
    // same effect as the "update_static_point_cloud_clusters" function below
    match process_ipc_content(args.ipc_args)
    {
        IPCUpdateResult::Success(i) =>
            {
                if let Some(new_lidar_file) = i.updated_lidar_file
                {
                    *args.time_since_update = Instant::now();
                    *args.current_content_file = new_lidar_file;
                }

                if let Some(num_points) = i.num_points
                {
                    *args.num_cloud_points = num_points;
                }

                *args.cluster_result_text = i.cluster_error_message;
            },
        IPCUpdateResult::Error(err) => *args.cluster_result_text = err,
        IPCUpdateResult::NoChange => {}
    }
}

/// Updates the point cloud clusters for a point cloud
///
/// `args` - struct holding the variables required to update a point cloud's clusters
pub fn update_point_cloud_clusters(args: HandleClusterUpdate)
{
    match launch_cluster_program(args.buffer_update_content, args.current_content_file)
    {
        Ok(_) =>
            {
                match read_cluster_output_file(args.buffer_update_content)
                {
                    Ok(colours) =>
                        {
                            args.buffer_groups.upload_instance_information(vec![UploadInformation
                            {
                                model_id: args.cube_model_id,
                                instance_translations: None,
                                instance_colours: Some(&colours)
                            }]);

                            *args.cluster_result_text = "Cluster program status: No Error".to_string();
                        },
                    Err(err) => *args.cluster_result_text = err,
                }
            }
        Err(err) => *args.cluster_result_text = err,
    }
}

/// Determines if the window should be closed due to the input of the user
///
/// `render_window` - the window being rendered
pub fn check_window_close(render_window: &mut RenderWindow)
{
    if render_window.get_key_input().iter().find(|x| **x == (Key::Escape, Action::Press)).is_some()
    {
        render_window.set_window_should_close(true);
    }
}

/// Updates the position of the point cloud from where it was originally placed
///
/// `render_variables` - struct holding the required variables for rendering
pub fn change_point_cloud_position(render_variables: &mut RenderData)
{
    let move_amount = 0.05;

    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F2, Action::Press)).is_some() ||
        render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F2, Action::Repeat)).is_some()
    {
        render_variables.cloud_translation.x += move_amount;
    }

    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F1, Action::Press)).is_some() ||
        render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F1, Action::Repeat)).is_some()
    {
        render_variables.cloud_translation.x -= move_amount;
    }

    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F4, Action::Press)).is_some() ||
        render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F4, Action::Repeat)).is_some()
    {
        render_variables.cloud_translation.z += move_amount;
    }

    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F3, Action::Press)).is_some() ||
        render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F3, Action::Repeat)).is_some()
    {
        render_variables.cloud_translation.z -= move_amount;
    }

    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F6, Action::Press)).is_some() ||
        render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F6, Action::Repeat)).is_some()
    {
        render_variables.cloud_translation.y += move_amount;
    }

    if render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F5, Action::Press)).is_some() ||
        render_variables.render_window.get_key_input().iter().find(|x| **x == (Key::F5, Action::Repeat)).is_some()
    {
        render_variables.cloud_translation.y -= move_amount;
    }
}

/// Updates the given variable to indicate if updates to the point cloud should be paused
///
/// `pause_updating` - variable holding whether or not to update the point cloud
/// `render_window` - the window being rendered to
pub fn check_pause_updates(pause_updating: &mut bool, render_window: &RenderWindow)
{
    if render_window.get_key_input().iter().find(|x| **x == (Key::P, Action::Press)).is_some()
    {
        *pause_updating = !*pause_updating;
    }
}

/// Writes the information about the scene to the window
///
/// `param` - the variables required to render scene information text
pub fn write_scene_info(param: TextWriteParam)
{
    param.text_renderer.update_window_dimensions(param.render_window.get_window_dimensions());
    param.text_renderer.buffer_text_for_rendering(format!("NP: {:.2}", (param.num_points as f32 / 1000.0)), vec2(0.025, 0.15), 30);

    if param.time_update.elapsed().as_secs() < 10
    {
        param.text_renderer.buffer_text_for_rendering(format!("TU:  {:.2}s", (param.time_update.elapsed().as_millis() as f32 / 1000.0)), vec2(0.025, 0.1), 30);
    }
    else
    {
        param.text_renderer.buffer_text_for_rendering("TU: > 10s", vec2(0.025, 0.1), 30);
    }
    param.text_renderer.buffer_text_for_rendering("MP:  ".to_string() + &param.camera.to_string_pos(), vec2(0.3, 0.15), 30);
    param.text_renderer.buffer_text_for_rendering("MD: ".to_string() + &param.camera.to_string_direction(), vec2(0.3, 0.1), 30);
    param.text_renderer.buffer_text_for_rendering(param.cluster_result_text, vec2(0.025, 0.025), 80);
    param.text_renderer.buffer_text_for_rendering("Epsilon: ".to_string() + &format!("{:.2}", param.epsilon), vec2(0.715, 0.025), 15);
    param.text_renderer.buffer_text_for_rendering("Min points: ".to_string() + &param.min_num_points.to_string(), vec2(0.85, 0.025), 15);
    param.view_fbos.buffer_write_fbo_information(param.text_renderer);
    param.text_renderer.render_buffered_text();
}