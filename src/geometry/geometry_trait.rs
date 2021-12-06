use nalgebra_glm::{TVec2, TVec3};

/// Required functions for a model to implement in order to be able to store its (non-instanced)
/// contents in GPU buffers
pub trait RenderableGeometry
{
    /// Number of bytes required for the model's vertices
    fn len_vertices_bytes(&self) -> isize;

    /// Number of bytes required for the model's indices
    fn len_indices_bytes(&self) -> isize;

    /// Number of bytes required for the model's texture coordinates
    fn len_tex_coords_bytes(&self) -> isize;

    /// Number of bytes required for the model's normals
    fn len_normals_bytes(&self) -> isize;

    /// Get the model's vertices
    fn get_vertices(&self) -> &Vec<TVec3<f32>>;

    /// Get the model's texture coordinates
    fn get_tex_coords(&self) -> &Vec<TVec2<f32>>;

    /// Get the model's normals
    fn get_normals(&self) -> &Vec<TVec3<f32>>;

    /// Get the model's indices
    fn get_indices(&self) -> &Vec<u32>;
}