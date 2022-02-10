use std::ffi::c_void;
use nalgebra_glm::{TMat4, TVec3, vec3};
use crate::rendering::camera::Camera;
use crate::gl_wrappers::shader_program_creation::ShaderProgram;
use crate::rendering::view_fbo::ViewFBO;
use crate::view_logic::view_selection::ViewSelection;
use crate::view_logic::view_transform::ViewTransformation;

/// Required variables to pass into the draw functions so that models can be rendered
#[derive(Copy, Clone)]
pub struct OutsideParam<'a>
{
    pub view_selection: &'a ViewSelection,
    pub view_fbos: &'a ViewFBO,
    pub window_resolution: (i32, i32),
    pub scene_matrix: &'a TMat4<f32>,
    pub camera: &'a Camera,
    pub cloud_translation: TVec3<f32>,
    pub reflect_vertical: i32
}

/// Provides information about what buffer ranges are needed to model a model
pub struct DrawCallInfo
{
    pub vertex_offset: i32,
    pub vertex_count: i32,
    pub indice_offset: *const c_void,
    pub indice_count: i32,
    pub instance_offset: u32,
    pub instance_count: i32,
}

pub type RenderFunction = fn(&ShaderProgram, &DrawCallInfo, OutsideParam);

/// Renders the cube model, which is used to represent points in the point cloud
pub fn cube_draw_function(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    create_shadow_map(shader_program, draw_call_info, outside_param);
    create_scene_side_views(shader_program, draw_call_info, outside_param);
    render_scene(shader_program, draw_call_info, outside_param);
}

/// Renders the plane model, which is used to represent the scene views
pub fn plane_draw_function(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    // The scene is rendered before this is called, which means that the viewport is not the full window.
    // The position of the views on the window is assuming the viewport is the entire screen
    unsafe{ gl::Viewport(0, 0, outside_param.window_resolution.0, outside_param.window_resolution.1) }
    draw_shadow_map(shader_program, draw_call_info, outside_param);
    draw_side_views(shader_program, draw_call_info, outside_param);
}

/// Renders the sun into the scene
pub fn draw_sun(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    shader_program.write_uint("drawingSun", 1);
    shader_program.write_vec3("sunPosition", &outside_param.view_fbos.get_sun_fbo().get_sun_position());
    shader_program.write_mat4("projViewMatrix", &outside_param.camera.get_projection_view_matrix());
    unsafe
        {
            gl::DrawElementsBaseVertex(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.vertex_offset);
        }

    shader_program.write_uint("drawingSun", 0);
}

/// Renders the arrow used to represent the point that the sun is looking at
pub fn draw_sun_arrow(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    shader_program.write_uint("drawingSunArrow", 1);
    shader_program.write_vec3("sunArrowPosition", &outside_param.view_fbos.get_sun_fbo().look_at_position());
    shader_program.write_float("sunArrowScale", 0.25); // Seemed like nice value
    shader_program.write_mat4("projViewMatrix", &outside_param.camera.get_projection_view_matrix());
    unsafe
        {
            gl::DrawElementsBaseVertex(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.vertex_offset);
            shader_program.write_vec3("sunArrowPosition", &vec3(0.0, 0.0, 0.0));
            shader_program.write_float("sunArrowScale", 1.0);
            gl::DrawArrays(gl::LINES, 0, 2);
        }

    shader_program.write_uint("drawingSunArrow", 0);
}

/// Renders the scene onto the window. Assumes the shadow map has been created
fn render_scene(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    let reset_viewport_x = ((outside_param.window_resolution.0 as f32) * 0.675) as i32;
    let reset_viewport_y = outside_param.window_resolution.1 as i32;

    let sun = outside_param.view_fbos.get_sun_fbo();

    sun.bind_draw_result();
    shader_program.write_int("reflectVertically", outside_param.reflect_vertical);
    shader_program.write_vec3("cloudTranslation", &outside_param.cloud_translation);
    shader_program.write_uint("drawingScene", 1);
    shader_program.write_mat4("lightPerspectiveMatrix", &sun.get_light_matrix());
    shader_program.write_mat4("projViewMatrix", &outside_param.camera.get_projection_view_matrix());
    shader_program.write_vec3("cameraPos", &outside_param.camera.get_position());
    shader_program.write_vec3("sunLightColour", &vec3(1.0, 1.0, 1.0));
    shader_program.write_vec3("sunDirection", &sun.get_sun_direction());

    unsafe
        {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::Viewport(0, ((outside_param.window_resolution.1 as f32 * 0.25)) as i32, reset_viewport_x, reset_viewport_y);
            gl::DrawElementsInstancedBaseVertexBaseInstance(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.instance_count, draw_call_info.vertex_offset, draw_call_info.instance_offset);
        }

    shader_program.write_uint("drawingScene", 0);
}

/// Creates the shadow map for the scene, which is only comprised of the point cloud points
fn create_shadow_map(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    let sun = outside_param.view_fbos.get_sun_fbo();
    sun.prepare_for_drawing(shader_program, &outside_param.scene_matrix, &outside_param.cloud_translation);
    unsafe
        {
            gl::DrawElementsInstancedBaseVertexBaseInstance(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.instance_count, draw_call_info.vertex_offset, draw_call_info.instance_offset);
        }
    sun.done_drawing(shader_program);
}

/// Creates the side view of the scene, which is only comprised of the point cloud points with no lighting
fn create_scene_side_views(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    let top_view = outside_param.view_fbos.get_top_fbo();
    let right_view = outside_param.view_fbos.get_right_fbo();

    shader_program.write_int("reflectVertically", outside_param.reflect_vertical);
    shader_program.write_uint("drawingFromSideView", 1);
    shader_program.write_mat4("rotationMatrix", &outside_param.scene_matrix);
    shader_program.write_mat4("projViewMatrix", &top_view.get_camera().get_projection_view_matrix());
    shader_program.write_vec3("cameraPos", &top_view.get_camera().get_position());
    shader_program.write_vec3("cloudTranslation", &outside_param.cloud_translation);

    top_view.bind_for_drawing();
    unsafe
        {
            gl::DrawElementsInstancedBaseVertexBaseInstance(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.instance_count, draw_call_info.vertex_offset, draw_call_info.instance_offset);
        }

    shader_program.write_mat4("projViewMatrix", &right_view.get_camera().get_projection_view_matrix());
    shader_program.write_vec3("cameraPos", &right_view.get_camera().get_position());
    right_view.bind_for_drawing();
    unsafe
        {
            gl::DrawElementsInstancedBaseVertexBaseInstance(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.instance_count, draw_call_info.vertex_offset, draw_call_info.instance_offset);
        }
    shader_program.write_uint("drawingFromSideView", 0);
}

/// Draws the shadow map as a view
fn draw_shadow_map(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    let sun = outside_param.view_fbos.get_sun_fbo();
    let view_selection = outside_param.view_selection;

    shader_program.write_uint("renderSideViews", 1);
    shader_program.write_mat4("projViewMatrix", &nalgebra_glm::identity());

    sun.bind_draw_result();
    shader_program.write_mat4("rotationMatrix", view_selection.get_shadow_view_transformation().get_transformation_matrix());

    unsafe
        {
            gl::DrawElementsBaseVertex(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.vertex_offset);
        }

    if view_selection.get_shadow_camera_view_selected() || view_selection.get_shadow_lookat_view_selected()
    {
        draw_view_outline(shader_program, draw_call_info, view_selection.get_shadow_view_transformation(), &view_selection.get_border_colour());
    }

    shader_program.write_uint("renderSideViews", 0);
}

/// Draws the side views of the scene onto the window
fn draw_side_views(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, outside_param: OutsideParam)
{
    let view_selection = outside_param.view_selection;
    let top_view = outside_param.view_fbos.get_top_fbo();
    let right_view = outside_param.view_fbos.get_right_fbo();

    shader_program.write_uint("renderSideViews", 2);
    top_view.bind_draw_result();
    shader_program.write_mat4("rotationMatrix", view_selection.get_top_view_transformation().get_transformation_matrix());

    unsafe
        {
            gl::DrawElementsBaseVertex(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.vertex_offset);
        }

    if view_selection.get_top_view_selected()
    {
        draw_view_outline(shader_program, draw_call_info, view_selection.get_top_view_transformation(), &view_selection.get_border_colour());
    }

    right_view.bind_draw_result();
    shader_program.write_mat4("rotationMatrix", view_selection.get_right_view_transformation().get_transformation_matrix());

    unsafe
        {
            gl::DrawElementsBaseVertex(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.vertex_offset);
        }

    if view_selection.get_right_view_selected()
    {
        draw_view_outline(shader_program, draw_call_info, view_selection.get_right_view_transformation(), &view_selection.get_border_colour());
    }

    shader_program.write_uint("renderSideViews", 0);
}

/// Draws the outline of the selected view
fn draw_view_outline(shader_program: &ShaderProgram, draw_call_info: &DrawCallInfo, view_transformation: &ViewTransformation, border_colour: &TVec3<f32>)
{
    unsafe
        {
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
            gl::StencilMask(0xFF);
        }

    shader_program.write_vec3("borderColour", border_colour);
    shader_program.write_uint("renderSideViewBorder", 1);
    shader_program.write_mat4("rotationMatrix", view_transformation.get_border_matrix());

    unsafe
        {
            gl::DrawElementsBaseVertex(gl::TRIANGLES, draw_call_info.indice_count, gl::UNSIGNED_INT, draw_call_info.indice_offset, draw_call_info.vertex_offset);
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
            gl::StencilMask(0xFF);
        }

    shader_program.write_uint("renderSideViewBorder", 0);
}