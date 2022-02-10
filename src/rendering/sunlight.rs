use glfw::{Action, Key};
use nalgebra_glm::normalize;
use nalgebra_glm::{TMat4, TVec3, vec3};
use crate::rendering::camera::{Camera, CameraType, OrthographicParam};
use crate::gl_wrappers::fbo::{FBO, TextureType};
use crate::gl_wrappers::shader_program_creation::ShaderProgram;
use crate::window::RenderWindow;

/// Represents a "logical" (as in the model is separate from this class) sun shining light onto the scene
pub struct SunLight
{
    fbo: FBO,
    look_at_position: TVec3<f32>,
    current_scroll_direction: ScrollDirection
}

enum ScrollDirection
{
    X,
    Y,
    Z
}

impl SunLight
{
    /// Creates a new sun. At this point the sun is over the world origin pointing downwards
    ///
    /// `window_dimensions` - the dimensions of the window being rendered to
    /// `binding_point` - texture unit to bind the sun's rendered perspective to
    pub fn new(window_dimensions: (i32, i32), binding_point: u32) -> SunLight
    {
        let camera_type = CameraType::Orthographic(OrthographicParam
        {
            left: -15.0,
            right: 15.0,
            bottom: -15.0,
            top: 15.0,
            near_plane: 0.1,
            far_plane: 100.0,
            position: vec3(0.0, 0.0, 0.0),
            direction: vec3(0.0, -1.0, 0.0),
            up: vec3(1.0, 0.0, 0.0)
        });

        let fbo = FBO::new(window_dimensions, binding_point, camera_type, TextureType::DepthComponent).unwrap();

        SunLight{ fbo, look_at_position: vec3(0.0, 0.0, 0.0), current_scroll_direction: ScrollDirection::X }
    }

    /// Sets the appropriate uniforms so that the sun's perspective can be rendered
    ///
    /// `shader_program` - the shader program used to render the sun
    pub fn prepare_for_drawing(&self, shader_program: &ShaderProgram, rotation_matrix: &TMat4<f32>, cloud_translation: &TVec3<f32>)
    {
        self.fbo.bind_for_drawing();

        shader_program.write_uint("drawingSceneLightPerspective", 1);
        shader_program.write_mat4("projViewMatrix", &self.fbo.get_projection_view_matrix());
        shader_program.write_mat4("rotationMatrix", &rotation_matrix);
        shader_program.write_vec3("cloudTranslation", cloud_translation);
    }

    /// Sets the required uniforms to indicate the sun is done drawing its perspective
    ///
    /// `shader_program` - the shader program used to render the sun
    pub fn done_drawing(&self, shader_program: &ShaderProgram)
    {
        shader_program.write_uint("drawingSceneLightPerspective", 0);
    }

    /// Get the projection-view matrix for the sun
    pub fn get_light_matrix(&self) -> TMat4<f32>
    {
        self.fbo.get_camera().get_projection_view_matrix()
    }

    /// Bind the FBO containing the rendered sun's perspective of the scene into the binding point
    /// given in the constructor
    pub fn bind_draw_result(&self)
    {
        self.fbo.bind_draw_result();
    }

    /// Get the position of the sun
    pub fn get_sun_position(&self) -> TVec3<f32>
    {
        self.fbo.get_camera().get_position()
    }

    /// Get the direction the sun is looking at
    pub fn get_sun_direction(&self) -> TVec3<f32>
    {
        self.fbo.get_camera().get_direction()
    }

    /// Set the position of the camera without key input
    ///
    /// `pos` - the position of the sun
    /// `centre_scene` - the centre of the scene that the sun should look at
    pub fn hard_set_sun_pos(&mut self, pos: TVec3<f32>, centre_scene: TVec3<f32>)
    {
        self.look_at_position = centre_scene;
        self.fbo.get_mut_camera().set_camera_pos(pos);
        self.fbo.get_mut_camera().point_camera_in_direction(normalize(&(self.look_at_position - pos)), false);
    }

    /// Get the string representation of the sun's position
    pub fn to_string_sun_position(&self, lidar_pos: TVec3<f32>) -> String
    {
        self.fbo.get_camera().to_string_pos(lidar_pos)
    }

    /// Get the string representation of the position the sun is looking at
    pub fn to_string_lookat_pos(&self, lidar_pos: TVec3<f32>) -> String
    {
        format!("{:.1}   {:.1}   {:.1}",
                self.look_at_position.x + lidar_pos.x,
                self.look_at_position.y + lidar_pos.y,
                self.look_at_position.z + lidar_pos.z)
    }

    /// Move the sun according to key input
    ///
    /// `render_window` - the structure representing the window being rendered to
    pub fn move_sun(&mut self, render_window: &RenderWindow)
    {
        Camera::update_camera_movement(&render_window,self.fbo.get_mut_camera());
        let sun_pos = self.get_sun_position();
        self.fbo.get_mut_camera().point_camera_in_direction(normalize(&(self.look_at_position - sun_pos)), false);
    }

    /// Get the position the sun is looking at
    pub fn look_at_position(&self) -> TVec3<f32>
    {
        self.look_at_position
    }

    /// Change the look-at position of the sun according to key input
    ///
    /// `render_window` - the structure representing the window being rendered to
    pub fn move_look_at_position(&mut self, render_window: &RenderWindow)
    {
        for x in render_window.get_scroll_history()
        {
            match self.current_scroll_direction
            {
                ScrollDirection::X => self.look_at_position.x += *x / 10.0,
                ScrollDirection::Y => self.look_at_position.y += *x / 10.0,
                ScrollDirection::Z => self.look_at_position.z += *x / 10.0,
            }
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::Num1, Action::Press)).is_some()
        {
            self.current_scroll_direction = ScrollDirection::X;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::Num2, Action::Press)).is_some()
        {
            self.current_scroll_direction = ScrollDirection::Y;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::Num3, Action::Press)).is_some()
        {
            self.current_scroll_direction = ScrollDirection::Z;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::W, Action::Press)).is_some() ||
            render_window.get_key_input().iter().find(|x| **x == (Key::W, Action::Repeat)).is_some()
        {
            self.look_at_position.x += 0.05;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::S, Action::Press)).is_some() ||
            render_window.get_key_input().iter().find(|x| **x == (Key::S, Action::Repeat)).is_some()
        {
            self.look_at_position.x -= 0.05;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::A, Action::Press)).is_some() ||
            render_window.get_key_input().iter().find(|x| **x == (Key::A, Action::Repeat)).is_some()
        {
            self.look_at_position.z -= 0.05;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::D, Action::Press)).is_some() ||
            render_window.get_key_input().iter().find(|x| **x == (Key::D, Action::Repeat)).is_some()
        {
            self.look_at_position.z += 0.05;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::Q, Action::Press)).is_some() ||
            render_window.get_key_input().iter().find(|x| **x == (Key::Q, Action::Repeat)).is_some()
        {
            self.look_at_position.y -= 0.05;
        }

        if render_window.get_key_input().iter().find(|x| **x == (Key::E, Action::Press)).is_some() ||
            render_window.get_key_input().iter().find(|x| **x == (Key::E, Action::Repeat)).is_some()
        {
            self.look_at_position.y += 0.05;
        }

        let sun_pos = self.get_sun_position();
        self.fbo.get_mut_camera().point_camera_in_direction(normalize(&(self.look_at_position - sun_pos)), false);
    }

    /// Clear the movement keys of the sun's camera, preventing further movement until additional
    /// appropriate keyboard is received
    pub fn clear_movement_key(&mut self)
    {
        self.fbo.get_mut_camera().clear_movement_key();
    }
}