mod args_parser;
mod geometry;
mod gl_wrappers;
mod helper_logic;
mod ipc_logic;
mod rendering;
mod view_logic;
mod view_port_constants;
mod window;

use std::time::Duration;
use glfw::{Action, Key};
use rendering::draw_functions::OutsideParam;
use helper_logic::initialization_functions::ProgramVariables;
use ipc_logic::ipc_content_logic::IPCProcessingArgs;
use helper_logic::main_loop_functions::*;

fn main()
{
    // This variable is never directly passed into the other functions, as that would allow unused members
    // to be modified. Hence verbose code below. All program variables condensed into one to reduce
    // main function length
    let mut program_variables = ProgramVariables::new();

    while !program_variables.render_data.render_window.should_close()
    {
        // ********** Respond to Key Inputs **********

        program_variables.render_data.render_window.poll_events();
        check_window_close(&mut program_variables.render_data.render_window);

        check_pause_updates(&mut program_variables.point_cloud_data.pause_updating, &program_variables.render_data.render_window);

        check_for_view_selection(&mut program_variables.render_data.view_selection, &mut program_variables.render_data.view_fbos,
                                 &mut program_variables.render_data.camera, &program_variables.render_data.render_window);

        update_camera_movement(&mut program_variables.render_data.view_selection, &mut program_variables.render_data.view_fbos,
                               &mut program_variables.render_data.camera, &program_variables.render_data.render_window);

        update_cluster_information(&mut program_variables.point_cloud_data.cluster_information,
                                   &mut program_variables.point_cloud_update.cluster_for_most_recent, &program_variables.render_data.render_window);

        change_point_cloud_position(&mut program_variables.render_data);

        reflect_point_cloud(&mut program_variables.render_data);

        add_lidar_pos(&mut program_variables.render_data);

        // ********** Update Clusters on Static Point Cloud **********

        if !program_variables.point_cloud_update.cluster_for_most_recent && program_variables.point_cloud_data.pause_updating
            && program_variables.render_data.render_window.get_key_input().iter().find(|x| **x == (Key::C, Action::Press)).is_some()
        {
            let cluster_update_args = HandleClusterUpdate
            {
                buffer_groups: &mut program_variables.render_data.buffer_groups,
                buffer_update_content: &program_variables.point_cloud_data.cluster_information,
                cube_model_id: program_variables.render_data.cube_model_id,
                cluster_result_text: &mut program_variables.point_cloud_data.cluster_result_text,
                current_content_file: &mut program_variables.point_cloud_update.current_content_file
            };

            update_point_cloud_clusters(cluster_update_args);
            program_variables.point_cloud_update.cluster_for_most_recent = true;
        }

        // ********** Update Point Cloud and Clusters **********

        if program_variables.args.using_file_ipc() && !program_variables.point_cloud_data.pause_updating
        {
            program_variables.point_cloud_update.cluster_for_most_recent = false;

            let ipc_processing_arg = IPCProcessingArgs
            {
                receiver: &program_variables.point_cloud_update.receiver,
                buffer_group: &mut program_variables.render_data.buffer_groups,
                point_model_id: program_variables.render_data.cube_model_id,
                cluster_information: &program_variables.point_cloud_data.cluster_information,
                display_lidar_pos: program_variables.args.display_lidar_pos
            };

            let ipc_update_args = HandleIPCUpdate
            {
                ipc_args: ipc_processing_arg,
                lidar_pos: &mut program_variables.point_cloud_data.position,
                num_cloud_points: &mut program_variables.point_cloud_data.num_points_cloud,
                time_since_update: &mut program_variables.point_cloud_data.time_since_update,
                cluster_result_text: &mut program_variables.point_cloud_data.cluster_result_text,
                current_content_file: &mut program_variables.point_cloud_update.current_content_file
            };

            update_point_cloud(ipc_update_args);
            program_variables.centre_views(program_variables.args.display_lidar_pos);
        }
        else if program_variables.args.using_file_ipc() && program_variables.point_cloud_data.pause_updating
        {
            if program_variables.render_data.render_window.get_key_input().iter().find(|x| **x == (Key::C, Action::Press)).is_some()
            {
                let cluster_update_args = HandleClusterUpdate
                {
                    buffer_groups: &mut program_variables.render_data.buffer_groups,
                    buffer_update_content: &program_variables.point_cloud_data.cluster_information,
                    cube_model_id: program_variables.render_data.cube_model_id,
                    cluster_result_text: &mut program_variables.point_cloud_data.cluster_result_text,
                    current_content_file: &mut program_variables.point_cloud_update.current_content_file
                };

                update_point_cloud_clusters(cluster_update_args);
            }
        }
        // ********** Render Scene + Views **********

        let outside_param = OutsideParam
        {
            view_selection: &program_variables.render_data.view_selection,
            view_fbos: &program_variables.render_data.view_fbos,
            camera: &program_variables.render_data.camera,
            window_resolution: program_variables.render_data.render_window.get_window_dimensions(),
            scene_matrix: &program_variables.render_data.translation_matrix,
            cloud_translation: program_variables.render_data.cloud_translation,
            reflect_vertical: program_variables.render_data.get_reflect_vertically()
        };
        program_variables.render_data.buffer_groups.render(outside_param);

        // ********** Render Information Text **********
        
        let text_param = TextWriteParam
        {
            text_renderer: &mut program_variables.render_data.text_renderer,
            view_fbos: &program_variables.render_data.view_fbos,
            camera: &mut program_variables.render_data.camera,
            time_update: &program_variables.point_cloud_data.time_since_update,
            render_window: &program_variables.render_data.render_window,
            num_points: program_variables.point_cloud_data.num_points_cloud,
            cluster_result_text: &program_variables.point_cloud_data.cluster_result_text,
            epsilon: program_variables.point_cloud_data.cluster_information.epsilon,
            min_num_points: program_variables.point_cloud_data.cluster_information.min_num_points,
            lidar_pos: program_variables.point_cloud_data.position,
            add_lidar_pos: program_variables.render_data.add_lidar_pos
        };
        write_scene_info(text_param);

        program_variables.render_data.render_window.swap_buffers();
    }

    program_variables.point_cloud_update.notify_cluster_thread_to_quit();

    // It should not take longer than twice the sleep duration for the cluster detection thread to notice
    // it is requested to quit. If it does take longer, it probably was not responsive anyways
    std::thread::sleep(Duration::from_millis(program_variables.args.sleep_duration_ms * 2));
}
