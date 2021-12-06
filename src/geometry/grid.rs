use std::mem::size_of;
use nalgebra_glm::{TVec2, TVec3, vec2, vec3};
use crate::geometry::geometry_trait::RenderableGeometry;

/// Represents the world grid in the scene to give a sense of scale to the pointcloud
pub struct Grid
{
    vertices: Vec<TVec3<f32>>,
    translations: Vec<TVec3<f32>>,
    colours: Vec<TVec3<f32>>,
    num_lines: i32,
    tex_coords: Vec<TVec2<f32>>,
    normals: Vec<TVec3<f32>>,
    indices: Vec<u32>
}

impl RenderableGeometry for Grid
{
    fn len_vertices_bytes(&self) -> isize
    {
        (self.vertices.len() * size_of::<TVec3<f32>>()) as isize
    }

    fn len_indices_bytes(&self) -> isize
    {
        0
    }

    fn len_tex_coords_bytes(&self) -> isize
    {
        (self.vertices.len() * size_of::<TVec2<f32>>()) as isize
    }

    fn len_normals_bytes(&self) -> isize
    {
        self.len_vertices_bytes()
    }

    fn get_vertices(&self) -> &Vec<TVec3<f32>>
    {
        &self.vertices
    }

    fn get_tex_coords(&self) -> &Vec<TVec2<f32>>
    {
        &self.tex_coords
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

impl Grid
{
    /// Creates a new grid with the given dimension
    ///
    /// `num_lines_per_dimension` - number of grid lines to extend in the x, y and z directions.
    ///                             Each grid line is separated by one world unit
    pub fn new(num_lines_per_dimension: i32) -> Grid
    {
        // Some really large value; unlikely point cloud will extend beyond this
        let max_offset_from_origin = 10_000_f32;

        let vertices = vec!
        [
            // X lines
            vec3(-max_offset_from_origin, 0.0, 0.0),
            vec3(0.0, 0.0, 0.0),

            vec3(0.0_f32, 0.0, 0.0),
            vec3(max_offset_from_origin, 0.0, 0.0),

            // Z lines
            vec3(0.0, 0.0, -max_offset_from_origin),
            vec3(0.0, 0.0, 0.0),

            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, max_offset_from_origin)
        ];

        let starting_value = num_lines_per_dimension / 2;

        // Pattern of specifying instances: Specify translation and colour of line directly on beginning
        // of negative dimension. Then specify negative dimension lines. Then specify information for
        // line on beginning of positive side of dimension. Then specify the positive dimension lines.

        // X-dimension lines

        let mut translations = vec![vec3(0.0, 0.0, 0.0)];
        let mut colours = vec![vec3(0.4, 0.0, 0.0)];

        for x in (-starting_value..starting_value).into_iter().filter(|x| *x != 0)
        {
            translations.push(vec3(0.0, 0.0, x as f32));
            colours.push(vec3(0.25_f32, 0.25, 0.25));
        }

        translations.push(vec3(0.0, 0.0, 0.0));
        colours.push(vec3(0.8, 0.0, 0.0));

        for x in (-starting_value..starting_value).into_iter().filter(|x| *x != 0)
        {
            translations.push(vec3(0.0, 0.0, x as f32));
            colours.push(vec3(0.25_f32, 0.25, 0.25));
        }

        // Z-dimension lines

        translations.push(vec3(0.0, 0.0, 0.0));
        colours.push(vec3(0.0, 0.0, 0.4));

        for z in (-starting_value..starting_value).into_iter().filter(|x| *x != 0)
        {
            translations.push(vec3(z as f32, 0.0, 0.0));
            colours.push(vec3(0.25_f32, 0.25, 0.25));
        }

        translations.push(vec3(0.0, 0.0, 0.0));
        colours.push(vec3(0.0, 0.0, 0.8));

        for z in (-starting_value..starting_value).into_iter().filter(|x| *x != 0)
        {
            translations.push(vec3(z as f32, 0.0, 0.0));
            colours.push(vec3(0.25_f32, 0.25, 0.25));
        }

        // Not used but provided so that indexing into layouts used by this model (the vertex layout)
        // is consistent for all non-instanced layouts
        let normals = vec![vec3(0.0, 0.0, 0.0); vertices.len()];
        let tex_coords = vec![vec2(0.0, 0.0); vertices.len()];

        Grid{ vertices, translations, colours, num_lines: num_lines_per_dimension, normals, tex_coords, indices: Vec::new() }
    }

    /// Get the instance translations for the grid lines
    pub fn get_translations(&self) -> &Vec<TVec3<f32>>
    {
        &self.translations
    }

    /// Get the instance colours for the grid lines
    pub fn get_colours(&self) -> &Vec<TVec3<f32>>
    {
        &self.colours
    }

    /// Get the number of lines that make up the grid
    pub fn get_num_instances(&self) -> i32
    {
        self.num_lines
    }
}