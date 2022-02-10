use std::ffi::c_void;
use std::mem::size_of;
use nalgebra_glm::{TVec2, TVec3, vec3};
use crate::gl_wrappers::buffer::{Buffer, BufferType};
use crate::geometry;
use crate::rendering::draw_functions::{DrawCallInfo, OutsideParam, RenderFunction};
use crate::geometry::geometry_trait::RenderableGeometry;
use crate::geometry::grid::Grid;
use crate::helper_logic::folder_location_functions::{get_point_models_folder, get_shaders_folder};
use crate::helper_logic::point_cloud_analyzer::InitialCloudAnalyzer;
use crate::gl_wrappers::shader_program_creation::{ShaderInitInfo, ShaderProgram, ShaderType};
use crate::gl_wrappers::vao::VAO;
use crate::rendering::draw_functions;

pub fn default_point_colour() -> TVec3<f32>
{
    vec3(0.0_f32, 0.7, 0.0)
}

/// Specifies how the geometrical information that makes up a model and how to render it
pub struct RenderInformation
{
    pub geometry: Box<dyn RenderableGeometry>,
    pub command: RenderFunction
}

/// Holds the requires elements needed to render the scene
pub struct SceneRenderer
{
    shader_program: ShaderProgram,
    grid: Grid,
    vao: VAO,

    vertices: Buffer,
    tex_coords: Buffer,
    normals: Buffer,

    instanced_translations: Buffer,
    instanced_colours: Buffer,

    indices: Buffer,


    base_number_instances: u32,
    current_instance_upload_index: u32,
    max_number_instances: u32,

    models: Vec<RenderInformation>,
    model_render_info: Vec<DrawCallInfo>,

}

/// Specifies the instance information for a model
pub struct UploadInformation<'a>
{
    pub model_id: ModelId,
    pub instance_translations: Option<&'a [TVec3<f32>]>,
    pub instance_colours: Option<&'a Vec<TVec3<f32>>>,
}

/// Unique identifier for a model
#[derive(Copy, Clone)]
pub struct ModelId
{
    id: usize
}

impl SceneRenderer
{
    /// Specifies all of the models and associated information needed to render a scene
    pub fn setup_scene_renderer(point_analyzer: &InitialCloudAnalyzer) -> (SceneRenderer, ModelId)
    {
        let mut scene_renderer_builder = SceneRendererBuilder::new();

        let cube_model_index = scene_renderer_builder.add_model(RenderInformation
        {
            geometry: Box::new( geometry::model::Model::from_file(get_point_models_folder().join("cube.obj"))),
            command: draw_functions::cube_draw_function,
        });

        scene_renderer_builder.add_model(RenderInformation
        {
            geometry: Box::new( geometry::model::Model::from_file(get_point_models_folder().join("sun.obj"))),
            command: draw_functions::draw_sun,
        });

        scene_renderer_builder.add_model(RenderInformation
        {
            geometry: Box::new( geometry::model::Model::from_file(get_point_models_folder().join("sunArrow.obj"))),
            command: draw_functions::draw_sun_arrow,
        });

        scene_renderer_builder.add_model(RenderInformation
        {
            geometry: Box::new( geometry::model::Model::from_file(get_point_models_folder().join("plane2.obj"))),
            command: draw_functions::plane_draw_function,
        });

        let mut scene_renderer = scene_renderer_builder.build(50_000);

        scene_renderer.upload_instance_information(vec!
        [
            UploadInformation
            {
                model_id: cube_model_index,
                instance_translations: Some(&point_analyzer.get_initial_points()),
                // By default the points in a scene will be a shade of green; personal preference
                instance_colours: Some(&vec![default_point_colour(); point_analyzer.get_initial_points().len()])
            }]);

        (scene_renderer, cube_model_index)
    }

    /// Creates a new scene renderer that with the ability to store and render instances that constitute a scene
    ///
    /// `models` - the models that make up a scene
    /// `max_number_instances` - maximum number of instances of all models combined in the scene
    fn new(models: Vec<RenderInformation>, max_number_instances: u32) -> SceneRenderer
    {
        let shader_program = create_shader_program();

        // 500 length is chosen as it is unlikely a point cloud will extend beyond this amount,
        // and at this length the edges of the grid are not visible
        let grid = Grid::new(500);
        let vertices_buffer_bytes = models.iter().map(|x| x.geometry.len_vertices_bytes()).sum::<isize>()
            + grid.len_vertices_bytes()
            + SceneRenderer::size_sun_arrow_bytes();

        let tex_coords_size_bytes = models.iter().map(|x|  x.geometry.len_tex_coords_bytes()).sum::<isize>()
            + grid.len_tex_coords_bytes()
            + SceneRenderer::size_sun_arrow_tex_bytes();

        let normals_buffer_bytes = models.iter().map(|x|  x.geometry.len_normals_bytes()).sum::<isize>()
            + grid.len_normals_bytes()
            + SceneRenderer::size_sun_arrow_bytes();

        let indices_buffer_bytes = models.iter().map(|x|  x.geometry.len_indices_bytes()).sum::<isize>() + grid.len_indices_bytes();

        let vao = VAO::new();
        vao.bind_vao();
        // These match up the layouts in the "sceneVertexShader.glsl" in the shaders folder
        vao.specify_index_layout(0, 3, gl::FLOAT, false, 0);
        vao.specify_index_layout(1, 2, gl::FLOAT, false, 0);
        vao.specify_index_layout(2, 3, gl::FLOAT, false, 0);
        vao.specify_index_layout(3, 3, gl::FLOAT, false, 0);
        vao.specify_index_layout(4, 3, gl::FLOAT, false, 0);

        vao.specify_divisor(3, 1);
        vao.specify_divisor(4, 1);

        let size_instance_buffer_bytes = (size_of::<TVec3<f32>>() * max_number_instances as usize) as isize;

        let mut buffer_group = SceneRenderer
        {
            shader_program,
            grid,
            vertices: Buffer::new(&vao, vertices_buffer_bytes, 1, BufferType::Array(0, 12)),
            tex_coords: Buffer::new(&vao, tex_coords_size_bytes, 1,BufferType::Array(1, 8)),
            normals: Buffer::new(&vao, normals_buffer_bytes, 1,BufferType::Array(2, 12)),
            instanced_translations: Buffer::new(&vao, size_instance_buffer_bytes, 1, BufferType::Array(4, 12)),
            instanced_colours: Buffer::new(&vao, size_instance_buffer_bytes, 1, BufferType::Array(3, 12)),
            indices: Buffer::new(&vao, indices_buffer_bytes, 1, BufferType::Indice),
            models,
            model_render_info: Vec::new(),
            max_number_instances,
            base_number_instances: 0,
            current_instance_upload_index: 0,
            vao,
        };

        buffer_group.upload_model_geometry();

        buffer_group
    }

    /// Uploads the model geometry into buffers and keeps track of the required indexing information
    /// into these buffers in order to render the uploaded geometry
    fn upload_model_geometry(&mut self)
    {
        /* All instanced layouts (translation and colours) are matched up to the per-vertex layouts.
          In other words:
          per-vertex layout:   [][][]
          per-instance layout: [][][]
          Above, the number of elements in the per-instance layout is the same as in the per-vertex.
          This is done even if no multi-instances of the per-vertex are rendered. This is done
          so that for the non-instanced rendered, the per-instance layout input (even if not used)
          is defined. This is done as it is not sure if doing otherwise is against OpenGL rules
        */

        let timeout = 5_000_000;

        let mut bytes_vertices_written = SceneRenderer::size_sun_arrow_bytes();
        let mut bytes_tex_coords_written = SceneRenderer::size_sun_arrow_tex_bytes();
        let mut bytes_normals_written = SceneRenderer::size_sun_arrow_bytes();
        let mut bytes_instanced_translations_written = (size_of::<TVec3<f32>>() * 2) as isize;
        let mut bytes_instanced_colours_written = (size_of::<TVec3<f32>>() * 2) as isize;
        let mut bytes_indices_written = 0;

        let num_vertices = self.grid.get_vertices().len();
        self.vertices.write_data_offset(self.grid.get_vertices(), &self.vao, timeout, bytes_vertices_written);
        self.tex_coords.write_data_offset( self.grid.get_tex_coords(), &self.vao, timeout, bytes_tex_coords_written);
        self.normals.write_data_offset( self.grid.get_normals(), &self.vao, timeout, bytes_normals_written);
        self.indices.write_data_offset( self.grid.get_indices(), &self.vao, timeout, bytes_indices_written);
        self.base_number_instances += num_vertices as u32;
        // By default no "effective" (0 values are considered to have no effect)
        // translations nor colours are given; any other values doesn't make sense
        self.instanced_translations.write_data_offset(&vec![vec3(0.0, 0.0, 0.0); num_vertices], &self.vao, timeout, bytes_instanced_translations_written);
        self.instanced_colours.write_data_offset(&vec![vec3(0.0, 0.0, 0.0); num_vertices], &self.vao, timeout, bytes_instanced_colours_written);

        bytes_vertices_written += self.grid.len_vertices_bytes();
        bytes_tex_coords_written += self.grid.len_tex_coords_bytes();
        bytes_normals_written += self.grid.len_normals_bytes();
        bytes_instanced_translations_written += (size_of::<TVec3<f32>>() * num_vertices) as isize;
        bytes_instanced_colours_written += (size_of::<TVec3<f32>>() * num_vertices) as isize;
        bytes_indices_written += self.grid.len_indices_bytes();

        let mut model_render_info = Vec::new();

        for render_info in &self.models
        {
            let num_vertices = render_info.geometry.get_vertices().len();

            self.vertices.write_data_offset(render_info.geometry.get_vertices(), &self.vao, timeout, bytes_vertices_written);
            self.tex_coords.write_data_offset( render_info.geometry.get_tex_coords(), &self.vao, timeout, bytes_tex_coords_written);
            self.normals.write_data_offset( render_info.geometry.get_normals(), &self.vao, timeout, bytes_normals_written);
            self.indices.write_data_offset( render_info.geometry.get_indices(), &self.vao, timeout, bytes_indices_written);

            self.instanced_translations.write_data_offset
            (&vec![vec3(0.0, 0.0, 0.0); num_vertices], &self.vao, timeout, bytes_instanced_translations_written);
            self.instanced_colours.write_data_offset
            (&vec![vec3(0.0, 0.0, 0.0); num_vertices], &self.vao, timeout, bytes_instanced_colours_written);

            let draw_call_info = DrawCallInfo
            {
                indice_offset: bytes_indices_written as *const c_void,
                indice_count: render_info.geometry.get_indices().len() as i32,
                instance_offset: (bytes_instanced_translations_written as usize / size_of::<TVec3<f32>>()) as u32,
                instance_count: num_vertices as i32,
                vertex_offset: (bytes_vertices_written as usize / size_of::<TVec3<f32>>()) as i32,
                vertex_count: num_vertices as i32,
            };

            model_render_info.push(draw_call_info);

            bytes_vertices_written += render_info.geometry.len_vertices_bytes();
            bytes_tex_coords_written += render_info.geometry.len_tex_coords_bytes();
            bytes_normals_written += render_info.geometry.len_normals_bytes();
            bytes_indices_written += render_info.geometry.len_indices_bytes();

            bytes_instanced_translations_written += (size_of::<TVec3<f32>>() * num_vertices) as isize;
            bytes_instanced_colours_written += (size_of::<TVec3<f32>>() * num_vertices) as isize;

            self.base_number_instances += num_vertices as u32;
        }

        self.model_render_info = model_render_info;

        self.current_instance_upload_index = self.base_number_instances;
    }

    /// Uploads the instance model of the specified models into GPU memory. If the sum of all instances
    /// exceeds the maximum specified in the scene renderer constructor, then excess instances will be discarded
    pub fn upload_instance_information(&mut self, info: Vec<UploadInformation>)
    {
        let timeout = 5_000_000;
        self.current_instance_upload_index = self.base_number_instances;

        let num_instances = self.grid.get_translations().len();
        let max_upload_amount = if self.current_instance_upload_index + num_instances as u32 > self.max_number_instances
        {
            let upload_amount = self.max_number_instances - self.current_instance_upload_index;
            eprintln!("Not enough VRam reserved to upload {} instances. Uploading: {}", num_instances, upload_amount);
            upload_amount
        }
        else
        {
            num_instances as u32
        };

        let bytes_offset = (self.current_instance_upload_index as usize * size_of::<TVec3<f32>>()) as isize;
        self.instanced_colours.write_data_offset(self.grid.get_colours(), &self.vao, timeout, bytes_offset);
        self.instanced_translations.write_data_offset(self.grid.get_translations(), &self.vao, timeout, bytes_offset);
        self.current_instance_upload_index += max_upload_amount;

        for x in info
        {
            // Theoretically (though not required) the number of elements for instance translations
            // and colours are the same
            let num_instances = match (x.instance_translations, x.instance_colours)
            {
                (Some(i), Some(_)) => i.len(),
                (Some(i), None) => i.len(),
                (None, Some(j)) => j.len(),
                _ => continue
            };

            let max_upload_amount = if self.current_instance_upload_index + num_instances as u32 > self.max_number_instances
            {
                let upload_amount = self.max_number_instances - self.current_instance_upload_index;
                eprintln!("Not enough VRam reserved to upload {} instances. Uploading: {}", num_instances, upload_amount);
                upload_amount
            }
            else
            {
                num_instances as u32
            };

            self.model_render_info[x.model_id.id].instance_count = max_upload_amount as i32;
            self.model_render_info[x.model_id.id].instance_offset = self.current_instance_upload_index;

            let bytes_offset = (self.current_instance_upload_index as usize * size_of::<TVec3<f32>>()) as isize;
            if let Some(colours) = x.instance_colours
            {
                self.instanced_colours.write_data_offset(colours, &self.vao, timeout, bytes_offset);
            }

            if let Some(translations) = x.instance_translations
            {
                self.instanced_translations.write_data_offset(translations, &self.vao, timeout, bytes_offset);
            }

            self.current_instance_upload_index += max_upload_amount;
        }
    }

    /// Renders the required scene onto the currently active frame buffer
    pub fn render(&mut self, outside_param: OutsideParam)
    {
        self.shader_program.use_program();
        self.vao.bind_vao();

        self.vertices.write_data_no_wait_no_binding
        (
            &vec![outside_param.view_fbos.get_sun_fbo().get_sun_position(),
                  outside_param.view_fbos.get_sun_fbo().look_at_position()], 0
        );

        // Models are rendered in the same order as specified in the constructor
        for (index, x) in self.models.iter().enumerate()
        {
            (x.command)(&self.shader_program, &self.model_render_info[index], outside_param)
        }

        self.shader_program.write_uint("drawingGrid", 1);
        self.shader_program.write_mat4("projViewMatrix", &outside_param.camera.get_projection_view_matrix());
        let reset_viewport_x = ((outside_param.window_resolution.0 as f32) * 0.675) as i32;
        let reset_viewport_y = outside_param.window_resolution.1 as i32;

        unsafe
            {
                gl::Viewport(0, (outside_param.window_resolution.1 as f32 * 0.25) as i32, reset_viewport_x, reset_viewport_y);

                let mut instance_offset: u32 = self.base_number_instances;
                gl::DrawArraysInstancedBaseInstance(gl::LINES, 2, 2, self.grid.get_num_instances(), instance_offset);
                instance_offset += self.grid.get_num_instances() as u32;
                gl::DrawArraysInstancedBaseInstance(gl::LINES, 4, 2, self.grid.get_num_instances(), instance_offset);
                instance_offset += self.grid.get_num_instances() as u32;
                gl::DrawArraysInstancedBaseInstance(gl::LINES, 6, 2, self.grid.get_num_instances(), instance_offset);
                instance_offset += self.grid.get_num_instances() as u32;
                gl::DrawArraysInstancedBaseInstance(gl::LINES, 8, 2, self.grid.get_num_instances(), instance_offset);
            }

        self.shader_program.write_uint("drawingGrid", 0);

        self.instanced_translations.update_fence();
        self.instanced_colours.update_fence();
    }

    /// Number of bytes required to store the sun arrow
    fn size_sun_arrow_bytes() -> isize
    {
        (size_of::<TVec3<f32>>() * 2) as isize
    }

    /// Number of bytes required to store the sun arrow texture coordinates
    fn size_sun_arrow_tex_bytes() -> isize
    {
        // This to make sure per-vertex layouts are of the same size; the sun arrow does not actually
        // have texture coordinates. Not sure if this has no relevance to OpenGL; done just in case
        (size_of::<TVec2<f32>>() * 2) as isize
    }
}

// This builder is not actually all that useful...it was useful in a previous iteration of the project.
// It's left as it doesn't add much redundant code and it's known that it does not cause any issue

/// Helps with the constructor of the scene renderer
struct SceneRendererBuilder
{
    models: Vec<RenderInformation>
}

impl SceneRendererBuilder
{
    /// Constructs a new empty scene renderer builder
    pub fn new() -> SceneRendererBuilder
    {
        SceneRendererBuilder { models: Vec::new() }
    }

    /// Adds a model to add to the scene renderer and gives a back a unique model id
    ///
    /// `model` - the model to be uploaded to the scene renderer
    pub fn add_model(&mut self, model: RenderInformation) -> ModelId
    {
        self.models.push(model);
        ModelId{ id: self.models.len() - 1 }
    }

    /// Creates a new scene renderer with the provided models
    pub fn build(self, max_number_instances: u32) -> SceneRenderer
    {
        SceneRenderer::new(self.models, max_number_instances)
    }
}

/// Creates a shader program that renderers the scene
fn create_shader_program() -> ShaderProgram
{
    let shader_program = ShaderProgram::new
        (
            vec!
            [
                ShaderInitInfo{ shader_type: ShaderType::Vertex, shader_location: get_shaders_folder().join("sceneVertexShader.glsl") },
                ShaderInitInfo{ shader_type: ShaderType::Fragment, shader_location: get_shaders_folder().join("sceneFragmentShader.glsl") },
            ]
        );
    shader_program.use_program();
    shader_program
}