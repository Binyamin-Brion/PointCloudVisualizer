use std::fs::File;
use std::io::{BufReader, Read};
use std::process::Command;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use lazy_static::lazy_static;
use nalgebra_glm::TVec3;
use crate::rendering::scene_renderer::{SceneRenderer, ModelId, UploadInformation, default_point_colour};
use crate::rendering::cluster_colour::ClusterColour;
use crate::helper_logic::folder_location_functions::get_cluster_program_location;
use crate::ipc_logic::ipc_receiver::SendContents;

// This is static so that it does need to be recalculated everytime the point cloud is updated though
// IPC. It could be passed in as a parameter, but the this variable is only used in one place and
// the structure holding program variables (and its substructures) are already big
lazy_static!
{
    static ref CLUSTER_COLOUR: ClusterColour = ClusterColour::new();
}

/// Holds required variables to perform cluster detection and read its results
#[derive(Clone)]
pub struct ClusterInformation
{
    pub output_file: String,
    pub epsilon: f32,
    pub min_num_points: u32,
}

/// Holds required variables to perform a multi-threaded IPC update
pub struct IPCProcessingArgs<'a>
{
    pub receiver: &'a Receiver<Result<SendContents, String>>,
    pub buffer_group: &'a mut SceneRenderer,
    pub point_model_id: ModelId,
    pub cluster_information: &'a ClusterInformation,
    pub display_lidar_pos: bool,
}

/// Holds information about the result of updating the point cloud
pub struct UploadResult
{
    pub updated_lidar_file: Option<String>,
    pub num_points: Option<usize>,
    pub lidar_pos: Option<TVec3<f32>>,
    pub cluster_error_message: String
}

/// The possible results of updating the point cloud
pub enum IPCUpdateResult
{
    Success(UploadResult),
    Error(String),
    NoChange
}

/// Updates the point cloud by using the IPC mechanism and uploading the results to the GPU for rendering
///
/// `ipc_args` - the variable required to use the IPC mechanism and upload results to GPU for rendering
pub fn process_ipc_content(ipc_args: IPCProcessingArgs) -> IPCUpdateResult
{
    match ipc_args.receiver.try_recv()
    {
        Ok(i) =>
            {
                match i
                {
                    Ok(i) =>
                        {
                            // If lidar pos is in the content file, the first data point is the lidar
                            // position. Thus the number of point cloud points instances is one less
                            //  than the number of points in the data file
                            let (num_instances, lidar_pos) = if ipc_args.display_lidar_pos
                            {
                                (i.points.len() - 1, Some(i.points[0]))
                            }
                            else
                            {
                                (i.points.len(), None)
                            };

                            let starting_index = i.points.len() - num_instances;

                            ipc_args.buffer_group.upload_instance_information(vec![UploadInformation
                            {
                                model_id: ipc_args.point_model_id,
                                instance_colours: Some(&vec![default_point_colour(); num_instances]),
                                instance_translations: Some(&i.points[starting_index..]),
                            }]);

                            return IPCUpdateResult::Success(UploadResult
                            {
                                updated_lidar_file: Some(i.file_name),
                                lidar_pos,
                                num_points: Some(num_instances),
                                cluster_error_message: "Cluster program status: No Error".to_string()
                            });
                        }
                    Err(err) =>  return IPCUpdateResult::Error(format!("Error parsing updated data: {}", err))
                }
            },
        Err(err) =>
            {
                if !err.to_string().contains("receiving on an empty channel")
                {
                    return IPCUpdateResult::Error(format!("Error in IPC communication: {}", err.to_string()));
                }
            }
    }

    IPCUpdateResult::NoChange
}

/// Launches the cluster program to find clusters in the point cloud
///
/// `cluster_information` - parameters for the cluster detection program
/// `content_file` - the file that contains the point cloud for the cluster detection
pub fn launch_cluster_program(cluster_information: &ClusterInformation, content_file: &String) -> Result<(), String>
{
    let cluster_output = Command::new(get_cluster_program_location())
        .arg(content_file)
        .arg(&cluster_information.output_file)
        .arg(cluster_information.epsilon.to_string())
        .arg(cluster_information.min_num_points.to_string())
        .output();

    match cluster_output
    {
        Ok(i) =>
            {
                match i.status.code()
                {
                    Some(code) =>
                        {
                            if code == -1
                            {
                                return Err("Error running cluster program :".to_string() + &String::from_utf8_lossy(&i.stderr));
                            }
                        },
                    None =>
                        {
                            return Err("Failed to get result of cluster detection program".to_string());
                        }
                }
            },
        Err(err) =>
            {
                return Err("Error with cluster detection program: ".to_string() + &err.to_string());
            }
    }

    Ok(())
}

/// Reads the result of the cluster detection and returns a vector of colours indicating the clusters
/// visually. An index of 0 in the return result corresponds to the first point in the point cloud file
/// passed to the cluster detection program.
///
/// `cluster_information` - the variable holding the location of the file holding the cluster detection result
pub fn read_cluster_output_file(cluster_information: &ClusterInformation) -> Result<Vec<TVec3<f32>>, String>
{
    let file = match File::open(&cluster_information.output_file)
    {
        Ok(i) => i,
        Err(err) => { return Err("Error opening cluster result file: ".to_string() + &err.to_string()); }
    };

    let mut buf_reader = BufReader::new(file);
    let mut file_contents = String::new();
    if buf_reader.read_to_string(&mut file_contents).is_err()
    {
        return Err("Failed to read cluster result file".to_string());
    }

    let mut colours = Vec::new();

    for x in file_contents.split_whitespace()
    {
        let cluster_index = match isize::from_str(x)
        {
            Ok(i) => i,
            Err(err) =>
                {
                    if cfg!(debug_assertions)
                    {
                        eprintln!("Could not convert {} to an integer: {}", x, err);
                    }
                    -1
                }
        };
        colours.push(CLUSTER_COLOUR.get_colour((cluster_index + 1) as usize));
    }

    Ok(colours)
}