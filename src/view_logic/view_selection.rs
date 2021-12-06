use glfw::{Action, MouseButton};
use nalgebra_glm::{TVec3, vec3};
use crate::view_logic::view_transform::ViewTransformation;
use crate::window::RenderWindow;

/// Handles the logic of determining if a view is selected or not
pub struct ViewSelection
{
    right_view: bool,
    shadow_map_camera: bool,
    shadow_map_lookat: bool,
    top_view: bool,
    border_colour: TVec3<f32>,

    top_view_transformation: ViewTransformation,
    right_view_transformation: ViewTransformation,
    shadow_map_view_transformation: ViewTransformation,
}

/// Represents the possible views that can be selected
enum ViewSelected
{
    Right,
    Shadow,
    Top,
}

/// Returns the vector to show a blue border around a selected view (only used for shadow map)
fn blue_colour() -> TVec3<f32> { vec3(0.0, 0.0, 0.5) }

/// Returns the vector to show a green border a selected view
fn green_colour() -> TVec3<f32> { vec3(0.0, 0.5, 0.0) }

impl ViewSelection
{
    /// Creates a new ViewSelection where all views are not selected
    pub fn new() -> ViewSelection
    {
        ViewSelection
        {
            right_view: false,
            shadow_map_camera: false,
            shadow_map_lookat: false,
            top_view: false,
            border_colour: green_colour(),

            // These values were based on trial and error; maybe something more formal could be done to get numbers,
            // but these work
            top_view_transformation:  ViewTransformation::new(vec3(0.675, 0.65, 0.0), vec3(0.3, 0.3, 0.0)),
            right_view_transformation: ViewTransformation::new(vec3(0.675, 0.0, 0.0), vec3(0.3, 0.3, 0.0)),
            shadow_map_view_transformation: ViewTransformation::new(vec3(0.675, -0.65, 0.0), vec3(0.3, 0.3, 0.0)),
        }
    }

    /// Determines if any of the views are selected
    pub fn is_any_view_selected(&self) -> bool
    {
        self.right_view ||
        self.top_view   ||
        self.shadow_map_camera ||
        self.shadow_map_lookat
    }

    /// Checks what view has been selected looking at user input, and then applies the logic to either
    /// select or deselect that view.
    ///
    /// `render_window` - the window that contains all user input
    pub fn update_view_selection(&mut self, render_window: &RenderWindow)
    {
        // The logic for selecting or deselecting is done in a different function so that that logic
        // can be tested- a OpenGL window is not created in a test, so passing in a render window
        // would not be possible

        if render_window.get_cursor_button_history().iter().find(|x| **x == (MouseButton::Button1, Action::Press)).is_some()
        {
            if self.right_view_transformation.cursor_over_view(render_window.get_latest_cursor_pos(), render_window.get_window_dimensions())
            {
                self.change_view_selection(Some(ViewSelected::Right));
            }
            else if self.shadow_map_view_transformation.cursor_over_view(render_window.get_latest_cursor_pos(), render_window.get_window_dimensions())
            {
                self.change_view_selection(Some(ViewSelected::Shadow));
            }
            else if self.top_view_transformation.cursor_over_view(render_window.get_latest_cursor_pos(), render_window.get_window_dimensions())
            {
                self.change_view_selection(Some(ViewSelected::Top));
            }
            else
            {
                self.change_view_selection(None);
            }
        }
    }

    /// Applies the logic of selecting or deselecting a view
    ///
    /// `view` - the view that was clicked on, if any
    fn change_view_selection(&mut self, view: Option<ViewSelected>)
    {
        // This is effectively a state machine. Could use State design pattern, but given how small
        // this state machine is, it may not be worthwhile. Also below code is known to work

        match view
        {
            Some(ViewSelected::Right) =>
                {
                    self.border_colour = green_colour();
                    self.right_view = !self.right_view;
                    self.top_view = false;
                    self.shadow_map_camera = false;
                    self.shadow_map_lookat = false;
                },
            Some(ViewSelected::Shadow) =>
                {
                    // Select to move the sun
                    if !self.shadow_map_camera && !self.shadow_map_lookat
                    {
                        self.border_colour = green_colour();
                        self.shadow_map_camera = true;
                        self.shadow_map_lookat = false;
                    }
                    // Select to move where the sun is looking at
                    else if self.shadow_map_camera && !self.shadow_map_lookat
                    {
                        self.border_colour = blue_colour();
                        self.shadow_map_camera = false;
                        self.shadow_map_lookat = true;
                    }
                    else
                    {
                        self.shadow_map_camera = false;
                        self.shadow_map_lookat = false;
                    }

                    // Border colour is not reset here to green as it will be set as needed
                    // when a view is selected
                    self.right_view = false;
                    self.top_view = false;
                },
            Some(ViewSelected::Top) =>
                {
                    self.border_colour = green_colour();
                    self.top_view = !self.top_view;
                    self.right_view = false;
                    self.shadow_map_camera = false;
                    self.shadow_map_lookat = false;
                },
            None =>
                {
                    self.top_view = false;
                    self.right_view = false;
                    self.shadow_map_camera = false;
                    self.shadow_map_lookat = false;
                }
        }
    }

    /// Check if the right view is selected
    pub fn get_right_view_selected(&self) -> bool { self.right_view }

    /// Check if the shadow camera view is selected (meaning move the sun's position)
    pub fn get_shadow_camera_view_selected(&self) -> bool { self.shadow_map_camera }

    /// Check if the shadow look at view is selected (meaning changing where the sun is looking at)
    pub fn get_shadow_lookat_view_selected(&self) -> bool { self.shadow_map_lookat }

    /// Check if the top view is selected
    pub fn get_top_view_selected(&self) -> bool { self.top_view }

    /// Get the transformation for the right view
    pub fn get_right_view_transformation(&self) -> &ViewTransformation { &self.right_view_transformation }

    /// Get the transformation for the shadow view
    pub fn get_shadow_view_transformation(&self) -> &ViewTransformation { &self.shadow_map_view_transformation }

    /// Get the transformation for the top view
    pub fn get_top_view_transformation(&self) -> &ViewTransformation { &self.top_view_transformation }

    /// Get the border colour to use for the selected view
    pub fn get_border_colour(&self) -> TVec3<f32>
    {
        self.border_colour
    }
}

#[cfg(test)]
mod tests
{
    use nalgebra_glm::TVec3;
    use crate::view_logic::view_selection::{blue_colour, green_colour, ViewSelected, ViewSelection};

    fn check_selected_invariants(view_selection: &ViewSelection, right_view: bool, shadow_camera: bool, shadow_lookat: bool, top_view: bool)
    {
        assert_eq!(right_view, view_selection.right_view);
        assert_eq!(shadow_camera, view_selection.shadow_map_camera);
        assert_eq!(shadow_lookat, view_selection.shadow_map_lookat);
        assert_eq!(top_view, view_selection.top_view);
    }

    fn check_border_colour(expected: TVec3<f32>, actual: TVec3<f32>)
    {
        // The values used for the border colour - 0.5 - can be stored exactly as a floating point value,
        // and they are never computed- the border colour is directly assigned, so these direct
        // comparisons are valid
        assert_eq!(expected.x, actual.x);
        assert_eq!(expected.y, actual.y);
        assert_eq!(expected.z, actual.z);
    }

    #[test]
    fn check_default_view_selection()
    {
        let view_selection = ViewSelection::new();
        check_selected_invariants(&view_selection, false, false, false, false);
        check_border_colour(green_colour(), view_selection.border_colour);
    }

    #[test]
    fn check_right_view_selected()
    {
        let mut view_selection = ViewSelection::new();

        view_selection.change_view_selection(Some(ViewSelected::Right));
        check_selected_invariants(&view_selection, true, false, false, false);
        check_border_colour(green_colour(), view_selection.border_colour);

        view_selection.change_view_selection(Some(ViewSelected::Right));
        check_selected_invariants(&view_selection, false, false, false, false);
        check_border_colour(green_colour(), view_selection.border_colour);
    }

    #[test]
    fn check_shadow_view_selected()
    {
        let mut view_selection = ViewSelection::new();

        view_selection.change_view_selection(Some(ViewSelected::Shadow));
        check_selected_invariants(&view_selection, false, true, false, false);
        check_border_colour(green_colour(), view_selection.border_colour);

        view_selection.change_view_selection(Some(ViewSelected::Shadow));
        check_selected_invariants(&view_selection, false, false, true, false);
        check_border_colour(blue_colour(), view_selection.border_colour);

        view_selection.change_view_selection(Some(ViewSelected::Shadow));
        check_selected_invariants(&view_selection, false, false, false, false);
        check_border_colour(blue_colour(), view_selection.border_colour);
    }

    #[test]
    fn check_top_view_selected()
    {
        let mut view_selection = ViewSelection::new();

        view_selection.change_view_selection(Some(ViewSelected::Top));
        check_selected_invariants(&view_selection, false, false, false, true);
        check_border_colour(green_colour(), view_selection.border_colour);

        view_selection.change_view_selection(Some(ViewSelected::Top));
        check_selected_invariants(&view_selection, false, false, false, false);
        check_border_colour(green_colour(), view_selection.border_colour);
    }

    #[test]
    fn check_no_view_selected()
    {
        let mut view_selection = ViewSelection::new();
        view_selection.change_view_selection(Some(ViewSelected::Right));
        view_selection.change_view_selection(None);
        check_selected_invariants(&view_selection, false, false, false, false);
        check_border_colour(green_colour(), view_selection.border_colour);
    }
}