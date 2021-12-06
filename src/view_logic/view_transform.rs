use nalgebra_glm::{TMat4, TVec3, vec3};

/// Holds the transformation to place a view in normalized device coordinates (NDC)
/// on the window being rendered to, as well as the logic for determining if
/// the cursor is over the view
pub struct ViewTransformation
{
    translation: TVec3<f32>,
    scale: TVec3<f32>,
    transformation_matrix: TMat4<f32>,
    border_matrix: TMat4<f32>,
}

impl ViewTransformation
{
    /// Creates a new ViewTransformation with a transformation matrix derived from the passed in
    /// parameters. All parameters are in NDC
    ///
    /// `translation` - where to place the view on the screen window
    /// `scale` - the size of the view on the screen window
    pub fn new(translation: TVec3<f32>, scale: TVec3<f32>) -> ViewTransformation
    {
        let mut transformation_matrix = nalgebra_glm::identity();
        transformation_matrix = nalgebra_glm::translate(&transformation_matrix, &translation);
        transformation_matrix = nalgebra_glm::scale(&transformation_matrix, &scale);

        let border_matrix = nalgebra_glm::scale(&transformation_matrix, &vec3(1.025, 1.025, 1.025));

        ViewTransformation { translation, scale, transformation_matrix, border_matrix }
    }

    /// Determines if the cursor position is over the view based off of its transformation
    ///
    /// `cursor_pos` - tuple indicating the x and y position of the cursor
    /// `window_dimensions` - the resolution of the window being rendered to
    pub fn cursor_over_view(&self, cursor_pos: (i32, i32), window_dimensions: (i32, i32)) -> bool
    {
        // These are not destructed in the function declaration in order to reduce the length of
        // the declaration
        let (cursor_x, cursor_y) = cursor_pos;
        let (win_x, win_y) = window_dimensions;

        // This works because in NDC, which the views are, the coordinates result in the view taking up
        // exactly the entire screen if no scaling is applied (model coordinates are all +- 1)
        let width_x = self.scale.x * win_x as f32;
        let width_y = self.scale.y * win_y as f32;

        // The 0.5 for win_x is because centre of screen in OpenGL is x-coordinate 0; in the created window it is
        // the left hand side of the screen
        let offset_translation_x = self.translation.x * win_x as f32 * 0.5;
        // The 0.5 for width_x is because view is centred around centre around screen, meaning from centre
        // half the view needs to be traversed to get to its left side
        let offset_scale_x = win_x as f32 * 0.5 - width_x * 0.5;
        let total_offset_x = (offset_translation_x + offset_scale_x) as i32;

        // The negative sign is because in OpenGL positive y goes upwards, whereas in the render window
        // it goes downwards
        let offset_translation_y = -(self.translation.y * win_y as f32 * 0.5);
        let offset_scale_y = win_y as f32 * 0.5 - width_y * 0.5;
        let total_offset_y = (offset_translation_y + offset_scale_y) as i32;

        total_offset_x <= cursor_x && cursor_x <= (total_offset_x + width_x as i32) &&
            total_offset_y <= cursor_y && cursor_y <= (total_offset_y + width_y as i32)
    }

    /// Get the transformation matrix of the view
    pub fn get_transformation_matrix(&self) -> &TMat4<f32>
    {
        &self.transformation_matrix
    }

    /// Get the transformation matrix of the border of the view
    pub fn get_border_matrix(&self) -> &TMat4<f32>
    {
        &self.border_matrix
    }
}

#[cfg(test)]
mod tests
{
    use nalgebra_glm::vec3;
    use crate::view_logic::view_transform::ViewTransformation;

    #[test]
    fn check_cursor_over_view()
    {
        let window_dimensions = (1000, 1000);
        let transformation = ViewTransformation::new(vec3(0.0, 0.0, 0.0), vec3(0.5, 0.5, 0.0));

        // The cursor will be over the view in position:
        // X: [250, 750]
        // Y: [250, 750]

        // Check if in the view
        assert!(transformation.cursor_over_view((500, 500), window_dimensions));
        assert!(transformation.cursor_over_view((750, 750), window_dimensions));
        assert!(transformation.cursor_over_view((250, 250), window_dimensions));

        // Check for outside of view
        assert!(!transformation.cursor_over_view((0, 0), window_dimensions));
        assert!(!transformation.cursor_over_view((1000, 1000), window_dimensions));
        assert!(!transformation.cursor_over_view((500, 1000), window_dimensions));
    }
}