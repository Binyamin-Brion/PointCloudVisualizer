use std::env;
use std::path::PathBuf;
use std::process::exit;

// The DevelopmentFlag indicates that this program is being run
// from the project folder. If this is not set, it is assumed
// that the program executable is being launched. This means
// that all required folders should be in the same folder
// as the executable

/// Get the location of the folder holding the bitmap font atlas
pub fn get_text_folder() -> PathBuf
{
    if env::var("DevelopmentFlag").is_ok()
    {
        get_root_project_folder().join("text_rendering")
    }
    else
    {
        PathBuf::new().join("text_rendering")
    }
}

/// Get the location of the folder containing the cluster
/// detection program
pub fn get_cluster_program_location() -> PathBuf
{
    if env::var("DevelopmentFlag").is_ok()
    {
        get_root_project_folder().join("ClusterDetectionExe/ReleaseBuild/ClusterDetectionExe")
    }
    else
    {
        PathBuf::new().join("ClusterDetectionExe/ClusterDetectionExe")
    }
}

/// Get the location of the shaders folder
pub fn get_shaders_folder() -> PathBuf
{
    if env::var("DevelopmentFlag").is_ok()
    {
        get_root_project_folder().join("shaders")
    }
    else
    {
        PathBuf::new().join("shaders")
    }
}

/// Get the location of the point models folder
pub fn get_point_models_folder() -> PathBuf
{
    if env::var("DevelopmentFlag").is_ok()
    {
        get_root_project_folder().join("point_models")
    }
    else
    {
        PathBuf::new().join("point_models")
    }
}

/// Get the location of hte folder holding the models
/// used in the program
fn get_root_project_folder() -> PathBuf
{
    let exe_location = match env::current_exe()
    {
        Ok(i) => i,
        Err(err) =>
            {
                eprintln!("Failed to get the location of the executable when finding shader folder. Info: {}", err.to_string());
                exit(-1);
            }
    };

    let configuration_folder = match exe_location.parent()
    {
        Some(i) => i,
        None =>
            {
                eprintln!("Failed to find the configuration_folder.");
                exit(-1);
            }
    };

    let target_folder = match configuration_folder.parent()
    {
        Some(i) => i,
        None =>
            {
                eprintln!("Failed to find the target_folder.");
                exit(-1);
            }
    };

    let project_folder = match target_folder.parent()
    {
        Some(i) => i,
        None =>
            {
                eprintln!("Failed to find the project_folder.");
                exit(-1);
            }
    };

    project_folder.to_path_buf()
}