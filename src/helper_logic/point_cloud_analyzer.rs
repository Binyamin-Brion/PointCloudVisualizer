use std::fs::File;
use std::io::Read;
use std::process::exit;
use nalgebra_glm::{TVec3, vec3};
use crate::ipc_logic::ipc_receiver::IPCContributor;

/// Holds information about the initial point cloud. This needed to render the initial point cloud
/// (when a static point cloud is being rendered) and to centre the cameras (both scene and views)
pub struct InitialCloudAnalyzer
{
    default_points: Vec<TVec3<f32>>,
    centre: TVec3<f32>,
    max_length: f32,
}

impl InitialCloudAnalyzer
{
    /// Reads the point cloud in the given file and extracts analytics from it. If no file is given,
    /// then an empty point cloud is assumed
    ///
    /// `initial_point_position` - file specifying the points of a point cloud
    pub fn new(initial_point_positions: &Option<String>) -> InitialCloudAnalyzer
    {
        match initial_point_positions
        {
            Some(i) =>
                {
                    let mut file = match File::open(&i)
                    {
                        Ok(i) => i,
                        Err(err) =>
                            {
                                eprintln!("Failed to open file: {}, with error: {}", i, err.to_string());
                                exit(-1);
                            }
                    };
                    let mut file_contents = String::new();
                    if let Err(err) = file.read_to_string(&mut file_contents)
                    {
                        if cfg!(debug_assertions)
                        {
                            println!("Failed to read initial point cloud file: {}", err.to_string());
                        }
                    }

                    let initial_points = IPCContributor::parse_read_data(&file_contents).unwrap();

                    // Find extremes of point cloud in each dimension
                    let mut min_x = f32::MAX;
                    let mut max_x = f32::MIN;
                    let mut min_z = f32::MAX;
                    let mut max_z = f32::MIN;
                    let mut min_y = f32::MAX;
                    let mut max_y = f32::MIN;

                    for point in &initial_points
                    {
                        min_x = min_x.min(point.x);
                        max_x = max_x.max(point.x);

                        min_z = min_z.min(point.z);
                        max_z = max_z.max(point.z);

                        min_y = min_y.min(point.y);
                        max_y = max_y.max(point.y);
                    }

                    let centre = vec3((min_x + max_x) / 2.0, (min_y + max_y) / 2.0, (min_z + max_z) / 2.0);
                    let max_length = (max_x - min_x).abs()
                                         .max((max_y - min_y).abs())
                                         .max((max_z - min_z).abs());

                    InitialCloudAnalyzer { default_points: initial_points, centre, max_length }
                },
            None => InitialCloudAnalyzer { default_points: vec![], centre: vec3(0.0, 0.0, 0.0), max_length: 0.0 }
        }
    }

    /// Get the points of the initial point cloud
    pub fn get_initial_points(&self) -> &Vec<TVec3<f32>>
    {
        &self.default_points
    }

    /// Get the centre of the initial point cloud
    pub fn get_centre(&self) -> TVec3<f32>
    {
        self.centre
    }

    /// Get the maximum length of the initial point cloud
    pub fn get_max_length(&self) -> f32
    {
        self.max_length
    }
}