use std::fs::File;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::str::FromStr;
use std::sync::mpsc::SyncSender;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use nalgebra_glm::{TVec3, vec3};
use crate::args_parser::IPCFiles;

/// Monitors the files used for updating the point cloud for any updated point cloud data
pub struct IPCContributor
{
    files: Vec<IPCFiles>,
    file_index: usize,
    sender: SyncSender<Result<SendContents, String>>,
    sleep_duration_ms: u64
}

/// The result of reading the output of the updated point cloud file
pub struct SendContents
{
    pub points: Vec<TVec3<f32>>,
    pub file_name: String,
}

impl IPCContributor
{
    /// Creates a new IPC monitor
    ///
    /// `ipc_files` - the files used for IPC
    /// `sender` - the variable used to send to the rest of the program (this variable runs in its own
    ///             thread) that new point cloud data is available
    pub fn new(ipc_files: Vec<IPCFiles>, sender: SyncSender<Result<SendContents, String>>, sleep_duration_ms: u64) -> IPCContributor
    {
        IPCContributor{ files: ipc_files, file_index: 0, sender, sleep_duration_ms }
    }

    /// Monitors the IPC files for updated point cloud data
    pub fn read_rendering_data(&mut self, quit_thread: &Mutex<bool>)
    {
        // Wait until the next file intended for updated data actually has updated point cloud data
        loop
        {
            match quit_thread.lock()
            {
                Ok(i) => if *i { return; },
                Err(err) => panic!("Failed to check if cluster thread should quit: {}", err)
            }

            let mut mutex_file = match File::open(&self.files[self.file_index].mutex_file_names)
            {
                Ok(i) => i,
                Err(_) =>
                    {
                        eprintln!("Failed to find file {}", self.files[self.file_index].mutex_file_names);
                        continue;
                    },
            };
            let mut mutex_file_contents = String::new();
            if let Err(err) = mutex_file.read_to_string(&mut mutex_file_contents)
            {
                if cfg!(debug_assertions)
                {
                    println!("Failed to read mutex file: {}", err.to_string());
                }

                return;
            }
            if mutex_file_contents == "taken"
            {
                break;
            }

            // This can be increased, but this is more of a debugging tool; analyzing clusters isn't
            // a quick process so more frequent updates does not help
            sleep(Duration::from_millis(self.sleep_duration_ms));
        }

        let mut point_cloud_data = String::new();
        {
            let mut point_cloud_file = File::open(&self.files[self.file_index].data_file_names).unwrap();
            if let Err(err) = point_cloud_file.read_to_string(&mut point_cloud_data)
            {
                if cfg!(debug_assertions)
                {
                    println!("Failed to read point cloud file: {}", err.to_string());
                }

                return;
            }
        }

        // Indicate file can now be used for further point cloud updates
        {
            let mut mutex_file = File::create(&self.files[self.file_index].mutex_file_names).unwrap();
            if let Err(err) = mutex_file.write(b"clear")
            {
               panic!("Failed to write to mutex file: {}", err.to_string());
            }
        }

        let send_result = match IPCContributor::parse_read_data(&point_cloud_data)
        {
            Ok(points) => self.sender.send(Ok(SendContents{ points, file_name: self.files[self.file_index].data_file_names.clone() })),
            Err(err) => self.sender.send(Err(err))
        };

        if let Err(err) = send_result
        {
            panic!("Failed to send the result of reading the initial point cloud file: {}", err.to_string());
        }

        self.file_index = (self.file_index + 1) % self.files.len();
    }

    /// Parses the data file containing the updated point cloud to extract the updated points of the
    /// point cloud
    ///
    /// `read_content` - the file containing updated point cloud data
    pub fn parse_read_data(read_content: &String) -> Result<Vec<TVec3<f32>>, String>
    {
        let handle_parsing = |vertex_number: usize, number: &str|
            {
                match f32::from_str(number)
                {
                    Ok(i) => Ok(i),
                    Err(err) =>
                        {
                            let error_result = format!("Failed to parse vertex number {} having value {}. Error: {}", vertex_number, number, err.to_string());
                            return Err(error_result)
                        }
                }
            };

        let pos_component_separator = "|";

        let mut split_content = Vec::from_iter(read_content.split(pos_component_separator));

        // In case the last character is the separator itself, remove it so that it is not interpreted
        // as part of a position
        if let Some(last_char) = split_content.last()
        {
            if *last_char == pos_component_separator || *last_char == ""
            {
                split_content.pop();
            }
        }

        let number_vertices = IPCContributor::round_number_down(split_content.len(), 3);

        if number_vertices != split_content.len()
        {
            eprintln!("Incomplete last vertex, did not receive three components to form a vertex. New vertex count: {}", number_vertices);
        }

        let mut parsed_vertices = Vec::new();

        for v in 0..number_vertices / 3
        {
            let x_coord = handle_parsing(v, split_content[v * 3])?;
            let y_coord = handle_parsing(v, split_content[v * 3 + 1])?;
            let z_coord = handle_parsing(v, split_content[v * 3 + 2])?;

            parsed_vertices.push(vec3(x_coord, z_coord, y_coord));
        }

        Ok(parsed_vertices)
    }

    /// Rounds the given number to the next lowest multiple provided
    ///
    /// `number_to_round` - the number to round to the next lowest multiple
    /// `multiple` - the multiple to round down to
    fn round_number_down(number_to_round: usize, multiple: usize) -> usize
    {
        if multiple == 0
        {
            return number_to_round;
        }

        let remainder = number_to_round % multiple;
        if remainder == 0
        {
            return number_to_round;
        }

        (number_to_round + multiple - remainder) - multiple
    }
}

#[cfg(test)]
mod tests
{
    use crate::ipc_logic::ipc_receiver::IPCContributor;

    #[test]
    fn parse_correct_num_vertices()
    {
        let string = "1|2|3|4|5|6";
        match IPCContributor::parse_read_data(&string.to_string())
        {
            Ok(i) =>
                {
                    assert_eq!(2, i.len(), "Incorrect number of parsed vertices");

                    assert_eq!(1 as f32, i[0].x);
                    assert_eq!(3 as f32, i[0].y);
                    assert_eq!(2 as f32, i[0].z);

                    assert_eq!(4 as f32, i[1].x);
                    assert_eq!(6 as f32, i[1].y);
                    assert_eq!(5 as f32, i[1].z);
                },
            Err(_) => assert!(false, "Failed to parse vertices")
        }
    }

    #[test]
    fn parse_correct_num_vertices_trailing_separator()
    {
        let string = "1|2|3|";
        match IPCContributor::parse_read_data(&string.to_string())
        {
            Ok(i) =>
                {
                    assert_eq!(1, i.len(), "Incorrect number of parsed vertices");

                    assert_eq!(1 as f32, i[0].x);
                    assert_eq!(3 as f32, i[0].y);
                    assert_eq!(2 as f32, i[0].z);
                },
            Err(_) => assert!(false, "Failed to parse vertices")
        }
    }

    #[test]
    fn parse_incorrect_num_vertices()
    {
        let string = "2|4|3|4";
        match IPCContributor::parse_read_data(&string.to_string())
        {
            Ok(i) =>
                {
                    assert_eq!(1, i.len(), "Incorrect number of parsed vertices");

                    assert_eq!(2 as f32, i[0].x);
                    assert_eq!(3 as f32, i[0].y);
                    assert_eq!(4 as f32, i[0].z);
                },
            Err(_) => assert!(false, "Failed to parse vertices")
        }
    }
}