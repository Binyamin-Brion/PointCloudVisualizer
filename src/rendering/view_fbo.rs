use nalgebra_glm::{TVec3, vec2, vec3};
use crate::rendering::camera::{Camera, CameraType, PerspectiveParam};
use crate::gl_wrappers::fbo::{FBO, TextureType};
use crate::rendering::sunlight::SunLight;
use crate::rendering::text_rendering::TextRendering;
use crate::view_logic::view_selection::ViewSelection;
use crate::window::RenderWindow;

/// Holds all of the FBOs required for the side views of the scene
pub struct ViewFBO
{
    right: FBO,
    top: FBO,
    sun: SunLight,
}

impl ViewFBO
{
    /// Creates a new top view, right view and sun view
    ///
    /// `render_window` - the window the views are going to be rendered to
    pub fn new(render_window: &RenderWindow) -> ViewFBO
    {
        ViewFBO
        {
            right: create_right_view_fbo(&render_window),
            top: create_top_view_fbo(&render_window),
            sun:  SunLight::new(render_window.get_window_dimensions(), 0)
        }
    }

    /// Move the position of the sun without regard to key input
    ///
    /// `centre_scene` - the centre of the scene (ie centre of the point cloud)
    pub fn hard_set_light_pos(&mut self, pos: TVec3<f32>, centre_scene: TVec3<f32>)
    {
        self.sun.hard_set_sun_pos(pos, centre_scene);
    }

    /// Move the position of the right view camera without regard to the key input
    ///
    /// `centre_scene` - the centre of the scene (ie centre of the point cloud)
    pub fn hard_set_right_view_pos(&mut self, pos: TVec3<f32>)
    {
        self.right.get_mut_camera().set_camera_pos(pos);
    }

    /// Move the position of the top view camera without regard to the key input
    ///
    /// `centre_scene` - the centre of the scene (ie centre of the point cloud)
    pub fn hard_set_top_view_pos(&mut self, pos: TVec3<f32>)
    {
        self.top.get_mut_camera().set_camera_pos(pos);
    }

    /// Update the position of the selected view, if any
    ///
    /// `view_selection` - structure holding the state of what view is selected
    /// `render_window` - the render window being rendered to
    pub fn update_camera_movement(&mut self, view_selection: &ViewSelection, render_window: &RenderWindow)
    {
        if view_selection.get_top_view_selected()
        {
            Camera::update_camera_movement(&render_window, &mut self.top.get_mut_camera());
        }
        else if view_selection.get_right_view_selected()
        {
            Camera::update_camera_movement(&render_window, &mut self.right.get_mut_camera());
        }
        else if view_selection.get_shadow_camera_view_selected()
        {
            self.sun.move_sun(&render_window);
        }
        else if view_selection.get_shadow_lookat_view_selected()
        {
            self.sun.move_look_at_position(&render_window);
        }
    }

    /// Buffers the held view information to be rendered (view positions, and for the sun, the direction
    /// of the camera in the sun view
    pub fn buffer_write_fbo_information(&self, text_renderer: &mut TextRendering)
    {
        text_renderer.buffer_text_for_rendering("RP: ".to_string() + &self.right.get_camera().to_string_pos(), vec2(0.55, 0.15), 30);
        text_renderer.buffer_text_for_rendering("TP:  " .to_string() + &self.top.get_camera().to_string_pos(), vec2(0.55, 0.1), 30);

        text_renderer.buffer_text_for_rendering("SP: ".to_string() + &self.sun.to_string_sun_position(), vec2(0.8, 0.15), 30);
        text_renderer.buffer_text_for_rendering("SD:  " .to_string() + &self.sun.to_string_lookat_pos(), vec2(0.8, 0.1), 30);
    }

    /// Reset the camera movement keys of all the views. All camera movements for the view will stop
    pub fn reset_movement_key_status(&mut self)
    {
        self.top.get_mut_camera().clear_movement_key();
        self.right.get_mut_camera().clear_movement_key();
        self.sun.clear_movement_key();
    }

    /// Get the reference to the right view FBO
    pub fn get_right_fbo(&self) -> &FBO { &self.right }

    /// Get the reference to the top view FBO
    pub fn get_top_fbo(&self) -> &FBO { &self.top }

    /// Get the reference to the sun view FBO
    pub fn get_sun_fbo(&self) -> &SunLight { &self.sun }
}

/// Creates the top view
///
/// `render_window` - the window being rendered to
fn create_top_view_fbo(render_window: &RenderWindow) -> FBO
{
    let top_view_camera_type = CameraType::Perspective(PerspectiveParam
    {
        window_dimensions: render_window.get_window_dimensions(),
        near_plane: 0.1,
        far_plane: 100.0,
        position: vec3(0.0, 0.0, 0.0),
        direction: vec3(0.0, -1.0, 0.0),
        up: vec3(1.0, 0.0, 0.0)
    });

    FBO::new(render_window.get_window_dimensions(), 0, top_view_camera_type, TextureType::RGB8).unwrap()
}

/// Creates the right view
///
/// `render_window` - the window being rendered to
fn create_right_view_fbo(render_window: &RenderWindow) -> FBO
{
    let right_view_camera_type = CameraType::Perspective(PerspectiveParam
    {
        window_dimensions: render_window.get_window_dimensions(),
        near_plane: 0.1,
        far_plane: 100.0,
        position: vec3(0.0, 0.0, 0.0),
        direction: vec3(0.0, 0.0, -1.0),
        up: vec3(0.0, 1.0, 0.0)
    });

    FBO::new(render_window.get_window_dimensions(), 0, right_view_camera_type, TextureType::RGB8).unwrap()
}
