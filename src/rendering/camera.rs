use glfw::{Action, MouseButton};
use nalgebra_glm::{cross, normalize, TMat4, TVec3};
use crate::window::RenderWindow;
use glfw::Key;

/// Representation of a camera through which the world is seen through
pub struct Camera
{
    view_matrix: TMat4<f32>,
    perspective_matrix: TMat4<f32>,
    movement_keys: [bool; 6],
    middle_key_down: bool,

    direction: TVec3<f32>,
    position: TVec3<f32>,
    up: TVec3<f32>,

    yaw: f32,
    pitch: f32,
    last_x: i32,
    last_y: i32,
    first_mouse: bool,
}

/// The direction that a camera should move in
#[repr(usize)]
pub enum MovementKeys
{
    Backward,
    Forward,
    Left,
    Right,
    UpForward,
    UpBackwards,
}

/// Required parameters to make an orthographic camera
pub struct OrthographicParam
{
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub position: TVec3<f32>,
    pub direction: TVec3<f32>,
    pub up: TVec3<f32>,
}

/// Required parameters to make a perspective camera
pub struct PerspectiveParam
{
    pub window_dimensions: (i32, i32),
    pub near_plane: f32,
    pub far_plane: f32,
    pub position: TVec3<f32>,
    pub direction: TVec3<f32>,
    pub up: TVec3<f32>,
}

/// Specifies what type of camera to create
pub enum CameraType
{
    Orthographic(OrthographicParam),
    Perspective(PerspectiveParam)
}

/// Updates the direction the camera should move in given the key input
macro_rules! camera_movement
{
    ($render_window: ident, $camera_obj: ident, $($key: expr, $movement_key: expr),+) =>
    {
        $(
            if $render_window.get_key_input().iter().find(|x| **x == ($key, Action::Press)).is_some()
            {
                $camera_obj.set_movement_key($movement_key, true);
            }
            else if $render_window.get_key_input().iter().find(|x| **x == ($key, Action::Release)).is_some()
            {
                $camera_obj.set_movement_key($movement_key, false);
            }
        ),+
    };
}

impl Camera
{
    /// Creates a new a camera that is created for the given window dimensions. The FOV is hard-coded
    /// to 45 degrees
    ///
    /// `camera_type` - the type of camera to create
    pub fn new(camera_type: CameraType) -> Camera
    {
        let view_matrix;
        let perspective_matrix;
        let direction;
        let position;
        let up;

        match camera_type
        {
            CameraType::Orthographic(i) =>
                {
                    view_matrix = nalgebra_glm::look_at(&i.position, &(i.position + i.direction), &i.up);
                    perspective_matrix = nalgebra_glm::ortho(i.left, i.right, i.bottom, i.top, i.near_plane, i.far_plane);

                    direction = i.direction;
                    position = i.position;
                    up = i.up;
                },
            CameraType::Perspective(i) =>
                {
                    view_matrix = nalgebra_glm::look_at(&i.position, &(i.position + i.direction), &i.up);
                    perspective_matrix = nalgebra_glm::perspective
                        (
                            (i.window_dimensions.0 as f32) / (i.window_dimensions.1 as f32),
                            45.0, i.near_plane, i.far_plane
                        );

                    direction = i.direction;
                    position = i.position;
                    up = i.up;
                }
        }

        let yaw = if direction.x == 0.0
        {
            // Special branch as in the else-branch, tan is not defined when the x-direction is
            // exactly zero
            if direction.z.is_sign_negative()
            {
                -90.0
            }
            else
            {
                90.0
            }
        }
        else
        {
            (direction.z / direction.x).atan()
        };

        Camera
        {
            view_matrix,
            perspective_matrix,
            direction,
            position,
            up,
            movement_keys: [false; 6],
            middle_key_down: false,
            yaw,
            pitch: direction.y.sin().to_degrees(),
            last_x: 0,
            last_y: 0,
            first_mouse: true,
        }
    }

    /// Sets the camera position and updates the view matrix
    ///
    /// `pos` - the new position of the camera
    pub fn set_camera_pos(&mut self, pos: TVec3<f32>)
    {
        self.position = pos;
        self.view_matrix = nalgebra_glm::look_at
            (
                &self.position,
                &(self.position + self.direction),
                &self.up,
            );
    }

    /// Return the projection * view matrix
    pub fn get_projection_view_matrix(&self) -> TMat4<f32>
    {
        self.perspective_matrix * self.view_matrix
    }

    pub fn get_position(&self) -> TVec3<f32>
    {
        self.position
    }

    /// Indicate that the camera should move in a given dimension
    pub fn set_movement_key(&mut self, key: MovementKeys, pressed: bool)
    {
        self.movement_keys[key as usize] = pressed;
    }

    /// Clears all movement keys, making the camera stop moving in all directions
    pub fn clear_movement_key(&mut self)
    {
        self.movement_keys[0] = false;
        self.movement_keys[1] = false;
        self.movement_keys[2] = false;
        self.movement_keys[3] = false;
    }

    /// Indicate that cursor movement should affect camera rotation
    pub fn set_rotation_button_status(&mut self, status: bool)
    {
        self.middle_key_down = status;
    }

    /// Call this when the camera stops processing camera movement for rotation, even if that
    /// pause is temporary
    pub fn reset_first_mouse(&mut self)
    {
        self.first_mouse = true;
    }

    /// Get the direction the camera is looking at
    pub fn get_direction(&self) -> TVec3<f32>
    {
        self.direction
    }

    /// Moves the camera in the given direction
    ///
    /// `render_window` - window that holds all user input
    /// `camera` - the instance of the camera that should have its position updated
    pub fn update_camera_movement(render_window: &RenderWindow, camera: &mut Camera)
    {
        camera_movement!(render_window, camera, Key::W, MovementKeys::Forward);
        camera_movement!(render_window, camera, Key::A, MovementKeys::Left);
        camera_movement!(render_window, camera, Key::S, MovementKeys::Backward);
        camera_movement!(render_window, camera, Key::D, MovementKeys::Right);
        camera_movement!(render_window, camera, Key::Q, MovementKeys::UpBackwards);
        camera_movement!(render_window, camera, Key::E, MovementKeys::UpForward);

        // Above macros set the movement flag. Below function actually moves the camera based off of
        // those flags. This split into two functions is for readability
        camera.update_camera_position();
    }

    /// Updates the rotation of the camera
    ///
    /// `render_window` - window that holds all user input
    /// `camera` - the instance of the camera that should be rotated
    pub fn update_camera_rotation(render_window: &RenderWindow, camera: &mut Camera)
    {
        if render_window.get_cursor_button_history().iter().find(|x| **x == (MouseButton::Button3, Action::Press)).is_some()
        {
            camera.set_rotation_button_status(true);
        }

        if render_window.get_cursor_button_history().iter().find(|x| **x == (MouseButton::Button3, Action::Release)).is_some()
        {
            camera.set_rotation_button_status(false);
            camera.reset_first_mouse();
        }

        // Actual rotation happens here. Another function does the actual rotation for readability
        camera.update_camera_rotate(render_window.get_cursor_history());
    }

    /// Get the string representation of the camera position
    pub fn to_string_pos(&self) -> String
    {
        format!("{:.1}   {:.1}   {:.1}", self.position.x, self.position.y, self.position.z)
    }

    /// Get the string representation of the direction the camera is looking at
    pub fn to_string_direction(&self) -> String
    {
        format!("{:.1}   {:.1}   {:.1}", self.direction.x, self.direction.y, self.direction.z)
    }

    /// Make the camera point in the given direction
    ///
    /// `direction` - the direction the camera should be looking in
    /// `set_direction` - true if the camera should, from this point onwards, consider the given
    ///                  direction "forward"
    pub fn point_camera_in_direction(&mut self, direction: TVec3<f32>, set_direction: bool)
    {
        // For the sun, it should change where it is looking at as it moves. However, the movement of
        // it should not change, as it introduces weird movement and shadows
        if set_direction
        {
            self.direction = direction;
        }

        self.view_matrix = nalgebra_glm::look_at
            (
                &self.position,
                &(self.position + direction),
                &self.up,
            );
    }

    /// Updates the camera position based off of the directions camera was specified to move in
    fn update_camera_position(&mut self)
    {
        let movement_scale = 0.05;

        if self.movement_keys[MovementKeys::Forward as usize]
        {
            self.position += self.direction * movement_scale;
        }

        if self.movement_keys[MovementKeys::Backward as usize]
        {
            self.position -= self.direction * movement_scale;
        }

        if self.movement_keys[MovementKeys::Left as usize]
        {
            self.position -= normalize(&cross(&self.direction, &self.up)) * movement_scale;
        }

        if self.movement_keys[MovementKeys::Right as usize]
        {
            self.position += normalize(&cross(&self.direction, &self.up)) * movement_scale;
        }

        if self.movement_keys[MovementKeys::UpBackwards as usize]
        {
            self.position -= self.up * movement_scale;
        }

        if self.movement_keys[MovementKeys::UpForward as usize]
        {
            self.position += self.up * movement_scale;
        }

        self.view_matrix = nalgebra_glm::look_at
            (
                &self.position,
                &(self.position + self.direction),
                &self.up,
            );
    }

    /// Rotate camera based off of cursor movement. If the camera's rotation button status is set
    /// to false (middle key is not pressed), this function has no effect
    ///
    /// `cursor_pos_history` - the locations of the cursor (typically of a single frame)
    fn update_camera_rotate(&mut self, cursor_pos_history: &Vec<(i32, i32)>)
    {
        if !self.middle_key_down
        {
            return;
        }

        for (x, y) in cursor_pos_history
        {
            if self.first_mouse
            {
                self.last_x = *x;
                self.last_y = *y;
                self.first_mouse = false;
            }

            let mut x_offset = (*x - self.last_x) as f32;
            let mut y_offset = (self.last_y - *y) as f32;

            x_offset *= 0.1;
            y_offset *= 0.1;

            self.last_x = *x;
            self.last_y = *y;

            self.yaw += x_offset;
            self.pitch += y_offset;

            if self.pitch > 89.0
            {
                self.pitch = 89.0;
            } else if self.pitch < -89.0
            {
                self.pitch = -89.0;
            }

            self.direction.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
            self.direction.y = self.pitch.to_radians().sin();
            self.direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

            self.direction = normalize(&self.direction);
        }
    }
}