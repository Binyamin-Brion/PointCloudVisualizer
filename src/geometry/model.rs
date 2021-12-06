use std::mem::size_of;
use std::path::PathBuf;
use std::process::exit;
use nalgebra_glm::{normalize, TVec2, TVec3, vec2, vec3};
use crate::geometry::geometry_trait::RenderableGeometry;

/// Holds data required to render a model
pub struct Model
{
    vertices: Vec<TVec3<f32>>,
    indices: Vec<u32>,
    normals: Vec<TVec3<f32>>,
    tex_coordinates: Vec<TVec2<f32>>,
}

impl RenderableGeometry for Model
{
    fn len_vertices_bytes(&self) -> isize
    {
        (self.vertices.len() * size_of::<TVec3<f32>>()) as isize
    }

    fn len_indices_bytes(&self) -> isize
    {
        (self.indices.len() * size_of::<u32>()) as isize
    }

    fn len_tex_coords_bytes(&self) -> isize
    {
        (self. tex_coordinates.len() * size_of::<TVec2<f32>>()) as isize
    }

    fn len_normals_bytes(&self) -> isize
    {
        (self.normals.len() * size_of::<TVec3<f32>>()) as isize
    }

    fn get_vertices(&self) -> &Vec<TVec3<f32>>
    {
        &self.vertices
    }

    fn get_tex_coords(&self) -> &Vec<TVec2<f32>>
    {
        &self.tex_coordinates
    }

    fn get_normals(&self) -> &Vec<TVec3<f32>>
    {
        &self.normals
    }

    fn get_indices(&self) -> &Vec<u32>
    {
        &self.indices
    }
}

impl Model
{
    /// Loads a model from the given file and extracts required rendering information
    pub fn from_file(file_location: PathBuf) -> Model
    {
        let model_options = tobj::LoadOptions
        {
            single_index: true,
            triangulate: true,
            ignore_points: false,
            ignore_lines: false
        };

        match tobj::load_obj(file_location.clone(), &model_options)
        {
            Ok((model, _)) =>
                {

                    let mut vertices = Vec::new();
                    let mut normals = Vec::new();
                    let mut tex_coordinates = Vec::new();
                    let mut indices = Vec::new();

                    for m in &model
                    {
                        let current_indice_count = vertices.len();

                        for i in &m.mesh.indices
                        {
                            indices.push(i + current_indice_count as u32);
                        }

                        for v in 0..m.mesh.positions.len() / 3
                        {
                            vertices.push(vec3(m.mesh.positions[3 * v], m.mesh.positions[3 * v + 1], m.mesh.positions[3 * v + 2]));
                        }

                        for n in 0..m.mesh.normals.len() / 3
                        {
                            normals.push(normalize(&vec3(m.mesh.normals[3 * n], m.mesh.normals[3 * n + 1], m.mesh.normals[3 * n + 2])));
                        }

                        for t in 0..m.mesh.texcoords.len() / 2
                        {
                            tex_coordinates.push(vec2(m.mesh.texcoords[t * 2], m.mesh.texcoords[t * 2 + 1]));
                        }
                    }

                    Model
                    {
                        vertices,
                        normals,
                        indices,
                        tex_coordinates
                    }
                },
            Err(err) =>
                {
                    eprintln!("Failed to load {:?}: {}", file_location, err.to_string());
                    exit(-1);
                }
        }
    }
}