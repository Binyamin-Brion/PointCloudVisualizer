use std::ffi::c_void;
use std::fmt::Debug;
use std::mem::size_of;
use std::process::exit;
use std::ptr::{copy_nonoverlapping, null};
use gl::types::GLsync;
use crate::gl_wrappers::vao::VAO;

/// Represents a buffer storage object on the GPU. It supports fast uploads by using a round-robin
/// synchronized iteration to find a suitable location in GPU memory to upload data to
pub struct Buffer
{
    buffers: Vec<u32>,
    pointers: Vec<*mut c_void>,
    fences: Vec<GLsync>,
    current_buffer_index: usize,
    number_buffers: usize,
    buffer_type: BufferType,
}

type BindingPoint = u32;
type Stride = i32;

/// Specifies the type of buffer to be created
pub enum BufferType
{
    Array(BindingPoint, Stride),
    Indice,
}

impl Buffer
{
    /// Creates a new buffer objects
    ///
    /// 'vao' - the vao that this buffer is a part of
    /// `size_buffer_bytes` - size of the buffer in bytes
    /// `number_buffers` - the number of buffers to use in the round-robin upload. The total vRAM used
    ///                     by the buffer is size_buffer_bytes * number_buffers
    /// 'buffer_type' - the type of buffer to create
    pub fn new(vao: &VAO, size_buffer_bytes: isize, number_buffers: usize, buffer_type: BufferType) -> Buffer
    {
        let mut buffers = Vec::new();
        let mut pointers = Vec::new();
        let mut fences = Vec::new();

        for _ in 0..number_buffers
        {
            let mut buffer: u32 = 0;
            let buffer_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
            let map_flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;

            unsafe
                {
                    gl::CreateBuffers(1, &mut buffer);
                    gl::NamedBufferStorage(buffer, size_buffer_bytes, null(), buffer_flags);

                    let ptr = gl::MapNamedBufferRange(buffer, 0, size_buffer_bytes, map_flags);

                    buffers.push(buffer);
                    pointers.push(ptr);
                    fences.push(gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0));
                }
        }

        let mut buffer = Buffer{ buffers, pointers, fences, current_buffer_index: 0, number_buffers, buffer_type };
        buffer.update_binding(vao);
        buffer
    }

    /// Write data to the buffer. If after waiting for the timeout provided the buffer is not free
    /// to be written to, an error aborts the program
    ///
    /// `data` - information to write to the buffer
    /// `vao' - the vao that the buffer is a part of
    /// `timeout` - the amount of time in nanoseconds to wait for the buffer to become free
    pub fn write_data<T: Debug>(&mut self, data: &Vec<T>, vao: &VAO, timeout: u64)
    {
        self.write_data_offset(data, vao, timeout, 0);
    }

    /// Write data to the buffer at an offset. If after waiting for the timeout provided the buffer is not free
    /// to be written to, an error aborts the program
    ///
    /// `data` - information to write to the buffer
    /// `vao' - the vao that the buffer is a part of
    /// `timeout` - the amount of time in nanoseconds to wait for the buffer to become free
    /// 'offset_bytes' - the offset into the buffer to write data to
    pub fn write_data_offset<T: Debug>(&mut self, data: &Vec<T>, vao: &VAO, timeout: u64, offset_bytes: isize)
    {
        let number_elements_offset = (offset_bytes as usize / size_of::<T>()) as isize;

        self.current_buffer_index = (self.current_buffer_index + 1) % self.number_buffers;
        self.wait_for_buffer(timeout);
        unsafe
            {
                copy_nonoverlapping(data.as_ptr(), (self.pointers[self.current_buffer_index] as *mut T).offset(number_elements_offset), data.len());
            }
        self.update_binding(vao);
    }

    /// Updates the buffer with the provided data without changing the binding of the vao (ie use the same
    /// buffer internally). No synchronization is done to ensure that the buffer is ready to be written to
    ///
    /// 'data' - the data to write to the buffer
    /// `offset_bytes` - the offset in bytes to the buffer to be written to
    pub fn write_data_no_wait_no_binding<T: Debug>(&mut self, data: &Vec<T>, offset_bytes: isize)
    {
        let number_elements_offset = (offset_bytes as usize / size_of::<T>()) as isize;
        unsafe
            {
                copy_nonoverlapping(data.as_ptr(), (self.pointers[self.current_buffer_index] as *mut T).offset(number_elements_offset), data.len());
            }
    }

    /// Updates the fence for the buffer object. This MUST be called after drawing operations that use
    /// the buffer are called
    pub fn update_fence(&mut self)
    {
        unsafe
            {
                gl::DeleteSync(self.fences[self.current_buffer_index]);
                self.fences[self.current_buffer_index] = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            }
    }

    /// Waits for the buffer to become available
    ///
    /// `timeout` - the amount of time in nanoseconds to wait for the buffer to become free
    fn wait_for_buffer(&mut self, timeout: u64)
    {
        unsafe
            {
                // First try without any flushing and no timeout, in case buffer is already free
                let wait_result =  gl::ClientWaitSync(self.fences[self.current_buffer_index], 0, 0);
                if wait_result == gl::ALREADY_SIGNALED || wait_result == gl::CONDITION_SATISFIED
                {
                    return;
                }

                // Buffer is not free, wait for the specified amount of time
                let wait_result = gl::ClientWaitSync(self.fences[self.current_buffer_index], 0, timeout);
                if wait_result == gl::ALREADY_SIGNALED || wait_result == gl::CONDITION_SATISFIED
                {
                    return;
                }

                // Buffer is still not free, hint to driver to make the buffer free by flushing commands, and wait again
                let wait_result = gl::ClientWaitSync(self.fences[self.current_buffer_index], gl::SYNC_FLUSH_COMMANDS_BIT, timeout);
                if wait_result == gl::ALREADY_SIGNALED || wait_result == gl::CONDITION_SATISFIED
                {
                    return;
                }

                // One last final attempt- execute all OpenGL commands issued earlier and then wait for the buffer to be free
                gl::Flush();
                let wait_result = gl::ClientWaitSync(self.fences[self.current_buffer_index], gl::SYNC_FLUSH_COMMANDS_BIT, timeout);
                if wait_result == gl::ALREADY_SIGNALED || wait_result == gl::CONDITION_SATISFIED
                {
                    return;
                }

                // TODO: Show error window to user
                eprintln!("Failed to wait for buffer: {}, {}", wait_result== gl::TIMEOUT_EXPIRED, wait_result== gl::WAIT_FAILED);
                exit(-1);
            }
    }

    /// Updates the binding of the VAO with the new buffer to render from
    ///
    /// 'vao' - the vao to update the binding of
    fn update_binding(&mut self, vao: &VAO)
    {
        match self.buffer_type
        {
            BufferType::Array(binding_point, stride) =>
                {
                    vao.update_vertex_buffer_binding(binding_point, self.buffers[self.current_buffer_index], 0, stride);
                },
            BufferType::Indice =>
                {
                    unsafe
                        {
                            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffers[self.current_buffer_index]);
                        }
                }
        }
    }
}
