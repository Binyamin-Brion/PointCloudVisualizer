use std::process::exit;
use clap::App;
use clap::{ArgMatches, load_yaml};

/// Holds the result of processing the arguments to the program
pub struct Args
{
    pub initial_data_model: Option<String>,
    pub ipc_files: Vec<IPCFiles>,
    pub display_lidar_pos: bool,
    pub sleep_duration_ms: u64
}

/// Specifies the files used for IPC
#[derive(Clone)]
pub struct IPCFiles
{
    pub mutex_file_names: String,
    pub data_file_names: String
}

impl Args
{
    /// Processes the arguments passed into the program
    pub fn parse_args() -> Args
    {
        let yaml = load_yaml!("../arguments.yml");
        let matches = App::from_yaml(yaml).get_matches();
        let mut args = Args
        {
            initial_data_model: None,
            ipc_files: vec![],
            display_lidar_pos: false,
            sleep_duration_ms: 250
        };

        Args::extract_validate_input(&matches, &mut args);
        args
    }

    /// Returns if the a static point cloud is being used (false) or if the point cloud is going to
    /// be updated using IPC (true)
    pub fn using_file_ipc(&self) -> bool
    {
        !self.ipc_files.is_empty()
    }

    /// Helper function for the constructor; determines if a static point cloud is being rendered
    /// (provided by initial point cloud file) or a dynamic point cloud (provided by IPC files)
    ///
    /// `matches` - the matches provided by the initial argument processing
    /// `args` - the structure to hold the result of the final argument processing in
    fn extract_validate_input(matches: &ArgMatches, args: &mut Args)
    {
        // Closure needed to use "?" operator; otherwise compiler will think it applies to extract_validate_input
        let str_to_string = |input: Option<&str>| Some(input?.to_string());
        args.initial_data_model = str_to_string(matches.value_of("render_initial_point_cloud"));

        match (matches.values_of("data_files"), matches.values_of("mutex_files"))
        {
            (Some(ipc), Some(mutex)) =>
                {
                    if ipc.len() != mutex.len()
                    {
                        eprintln!("Must have the same number of ipc and mutex files");
                        exit(-1);
                    }

                    for (ipc_file, mutex_file) in ipc.into_iter().zip(mutex.into_iter())
                    {
                        args.ipc_files.push(IPCFiles{ mutex_file_names: mutex_file.to_string(), data_file_names: ipc_file.to_string() })
                    }
                }
            _ =>
                {
                    if matches.value_of("render_initial_point_cloud").is_none()
                    {
                        eprintln!("No work specified for the program. Must specify IPC files \
                        and/or a file containing point cloud data to render");
                        exit(-1);
                    }
                }
        }

        if let Some(use_lidar_pos) = matches.value_of("display_lidar_pos")
        {
            // As mentioned in arguments.yml, not sure why clap requires a value for an arg. If a value
            // is being added, may as well give user an option if -p arg has any effect, even if arg
            // itself is not required
            match use_lidar_pos.parse::<u64>()
            {
                Ok(i) => args.display_lidar_pos = i != 0,
                Err(err) =>
                    {
                        eprintln!("Invalid number for the display lidar pos option: {}. Error: {}", use_lidar_pos, err);
                        exit(-1);
                    }
            }
        }

        if let Some(wait_duration) = matches.value_of("sleep_duration")
        {
            match wait_duration.parse::<u64>()
            {
                Ok(i) => args.sleep_duration_ms = i,
                Err(err) =>
                    {
                        eprintln!("Invalid number for the sleep period: {}. Error: {}", wait_duration, err);
                        exit(-1);
                    }
            }
        }
    }
}