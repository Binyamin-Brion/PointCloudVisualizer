/// Abstraction of a VAO object
pub struct VAO
{
    vao: u32
}

impl VAO
{
    /// Creates a new VAO. The VAO is not binded
    pub fn new() -> VAO
    {
        let mut vao: u32 = 0;

        unsafe
            {
                gl::GenVertexArrays(1, &mut vao);
            }

        VAO{ vao }
    }

    /// Subsequent operations to buffer layouts will be stored  in this instance of the VAO
    pub fn bind_vao(&self)
    {
        unsafe
            {
                gl::BindVertexArray(self.vao);
            }
    }

    /// Set the divisor for an index. The VAO must have been binded before calling this function
    pub fn specify_divisor(&self, index: u32, divisor: u32)
    {
        unsafe
            {
                gl::VertexBindingDivisor(index, divisor);
            }
    }

    /// Specify how to interpret a portion of a buffer for rendering
    pub fn specify_index_layout(&self, index: u32, count: i32, data_type: u32, normalized: bool, relative_offset: u32)
    {
        unsafe
            {
                gl::VertexAttribFormat(index, count, data_type, normalized as u8, relative_offset);
                gl::EnableVertexAttribArray(index);
            }
    }

    /// Specify what portion of a buffer to use for rendering
    pub fn update_vertex_buffer_binding(&self, index: u32, buffer: u32, offset: isize, stride: i32)
    {
        unsafe
            {
                gl::VertexArrayVertexBuffer(self.vao, index, buffer, offset, stride);
            }
    }
}

impl Drop for VAO
{
    fn drop(&mut self)
    {
        unsafe
            {
                gl::DeleteVertexArrays(1, &self.vao);
            }
    }
}