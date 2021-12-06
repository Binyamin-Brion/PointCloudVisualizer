use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use std::ptr::{null, null_mut};
use nalgebra_glm::{TMat4, TVec2, TVec3};

/// Abstraction of a shader program
pub struct ShaderProgram
{
    shader_program: u32
}

/// Abstraction of a shader
struct Shader
{
    shader: u32,
}

/// Information bundle used to create and initialize an OpenGL shader
#[derive(Debug)]
pub struct ShaderInitInfo
{
    pub shader_type: ShaderType,
    pub shader_location: PathBuf,
}

/// The possible types of shaders supported
// Note: All shaders types may not be listed- only those that will be used in a program are listed
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShaderType
{
    Fragment = gl::FRAGMENT_SHADER,
    Vertex = gl::VERTEX_SHADER,
}

impl ShaderProgram
{
    /// Creates a new shader program using the shaders that will be created from the given shader information
    ///
    /// `shaders` - information to create shaders used in the creation of the shader program
    pub fn new(shaders: Vec<ShaderInitInfo>) -> ShaderProgram
    {
        ShaderProgram::check_validate_shader_info(&shaders);

        let mut created_shaders = Vec::new();
        let shader_program: u32;
        unsafe
            {
                shader_program = gl::CreateProgram();

                for x in shaders
                {
                    let shader = ShaderProgram::create_shader(&x);
                    gl::AttachShader(shader_program, shader.shader);
                    created_shaders.push(shader);
                }

                gl::LinkProgram(shader_program);

                for x in created_shaders
                {
                    // This is required if the DetachShader called in the Shader drop function is
                    // to have any effect
                    gl::DetachShader(shader_program, x.shader);
                }
            }

        ShaderProgram::check_shader_program_linkage(shader_program);
        ShaderProgram{ shader_program }
    }

    /// Uploads the given integer to the uniform of the specified name
    ///
    /// `uniform_name` - name of the uniform to upload the integer to
    /// `data` - the integer to upload to the uniform
    pub fn write_int<A: AsRef<str>>(&self, uniform_name: A, data: i32)
    {
        unsafe
            {
                gl::Uniform1i(self.get_uniform_location(uniform_name.as_ref()), data);
            }
    }

    /// Uploads the given unsigned integer to the uniform of the specified name
    ///
    /// `uniform_name` - name of the uniform to upload the unsigned integer to
    /// `data` - unsigned integer to upload to the uniform
    pub fn write_uint<A: AsRef<str>>(&self, uniform_name: A, data: u32)
    {
        unsafe
            {
                gl::Uniform1ui(self.get_uniform_location(uniform_name.as_ref()), data);
            }
    }

    /// Uploads the given 3d vector of floats to the uniform of the specified name
    ///
    /// `uniform_name` - name of the uniform to upload the 3d vector of floats to
    /// `data` - 3d vector of floats to upload to the uniform
    pub fn write_vec3<A: AsRef<str>>(&self, uniform_name: A, data: &TVec3<f32>)
    {
        unsafe
            {
                gl::Uniform3fv(self.get_uniform_location(uniform_name.as_ref()), 1, data.as_ptr());
            }
    }

    /// Uploads the float to the uniform of the specified name
    ///
    /// `uniform_name` - name of the uniform to upload the float to
    /// `data` - float to upload to the uniform
    pub fn write_float<A: AsRef<str>>(&self, uniform_name: A, data: f32)
    {
        unsafe
            {
                gl::Uniform1f(self.get_uniform_location(uniform_name.as_ref()), data);
            }
    }

    /// Uploads the given 2d vector of floats to the uniform of the specified name
    ///
    /// `uniform_name` - name of the uniform to upload the 2d vector of floats to
    /// `data` - 2d vector of floats to upload to the uniform
    pub fn write_vec2<A: AsRef<str>>(&self, uniform_name: A, data: &TVec2<f32>)
    {
        unsafe
            {
                gl::Uniform2fv(self.get_uniform_location(uniform_name.as_ref()), 1, data.as_ptr());
            }
    }

    /// Uploads the given matrix of floats to the uniform of the specified name
    ///
    /// `uniform_name` - name of the uniform to upload the matrix to
    /// `data` - 4x4 matrix of floats to upload to the uniform
    pub fn write_mat4<A: AsRef<str>>(&self, uniform_name: A, data: &TMat4<f32>)
    {
        unsafe
            {
                gl::UniformMatrix4fv(self.get_uniform_location(uniform_name.as_ref()), 1, gl::FALSE, data.as_ptr());
            }
    }

    /// Use the program for subsequent draw operations
    pub fn use_program(&self)
    {
        unsafe
            {
                gl::UseProgram(self.shader_program);
            }
    }

    fn get_uniform_location(&self, uniform_name: &str) -> i32
    {
        let uniform_c_string = CString::new(uniform_name).unwrap();
        unsafe{ gl::GetUniformLocation(self.shader_program, uniform_c_string.as_ptr()) }
    }

    /// Check if the given shader information is sufficient to create a shader program
    ///
    /// `shaders` - all of the information required to create shaders for a shader program
    fn check_validate_shader_info(shaders: &Vec<ShaderInitInfo>)
    {
        let number_vertex_shaders = shaders.iter().filter(|x| x.shader_type == ShaderType::Vertex).count();

        // At the very least a shader program must have:
        // * Exactly 1 vertex shader
        // * Exactly 1 fragment shader

        if number_vertex_shaders == 0
        {
            eprintln!("No vertex shader specified. Aborting.");
            exit(-1);
        }
        else if number_vertex_shaders > 1
        {
            eprintln!("Too many vertex shaders specified (num = {}). Aborting.", number_vertex_shaders);
            exit(-1);
        }

        let number_frag_shaders = shaders.iter().filter(|x| x.shader_type == ShaderType::Fragment).count();

        if number_frag_shaders == 0
        {
            eprintln!("No fragment shader specified. Aborting.");
            exit(-1);
        }
        else if number_frag_shaders > 1
        {
            eprintln!("Too many vertex shaders specified (num = {}). Aborting.", number_frag_shaders);
            exit(-1);
        }

        let number_geometry_shaders = shaders.iter().filter(|x| x.shader_type == ShaderType::Fragment).count();

        if number_geometry_shaders > 1
        {
            eprintln!("Too many geometry shaders specified (num = {}). Aborting.", number_geometry_shaders);
            exit(-1);
        }
    }

    /// Creates an shader from the given initialization information
    ///
    /// `shader_info` - information required to create a shader
    fn create_shader(shader_info: &ShaderInitInfo) -> Shader
    {
        let shader: u32;
        unsafe
            {
                shader = gl::CreateShader(shader_info.shader_type as u32);

                let shader_content = ShaderProgram::read_file(&shader_info.shader_location);
                let shader_content_cstr = CString::from_vec_unchecked(shader_content.as_bytes().to_owned());

                gl::ShaderSource(shader, 1, &shader_content_cstr.as_ptr(), null());
                gl::CompileShader(shader);

                if let Some(error_string) = ShaderProgram::check_shader_compilation(shader)
                {
                    // TODO Implement proper display formatting
                    eprintln!("Failed to compile shader {:?}. Info: {}", shader_info, error_string);
                    exit(-1);
                }
            }

        Shader{ shader }
    }

    /// Determine if the shader source code is valid GLSL
    ///
    /// `shader` - the shader for which to check compilation for
    fn check_shader_compilation(shader: u32) -> Option<String>
    {
        let mut success = 1;

        unsafe
            {
                gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

                if success == 0
                {
                    let mut error_message_length = 0;
                    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut error_message_length);

                    let error_buffer = vec![' ' as u8; error_message_length as usize + 1];
                    let error_string = CString::from_vec_unchecked(error_buffer);

                    gl::GetShaderInfoLog(shader, error_message_length, null_mut(), error_string.as_ptr() as *mut gl::types::GLchar);

                    return Some(error_string.to_string_lossy().into_owned());
                }
            }

        None
    }

    /// Checks that the shader program was successfully created
    ///
    /// `shader_program` - the shader program to check for linkage
    fn check_shader_program_linkage(shader_program: u32)
    {
        let mut success = 1;

        unsafe
            {
                gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);

                if success == 0
                {
                    let mut error_message_length = 0;
                    gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut error_message_length);

                    let error_buffer = vec![' ' as u8; error_message_length as usize + 1];
                    let error_string = CString::from_vec_unchecked(error_buffer);

                    gl::GetProgramInfoLog(shader_program, error_message_length, null_mut(), error_string.as_ptr() as *mut gl::types::GLchar);

                    eprintln!("Failed to link shader program. Got the following error: {}", error_string.to_string_lossy().into_owned());
                    exit(-1);
                }
            }
    }

    /// Read the file containing the shader source code
    ///
    /// `file_location` - path to the file containing the shader source code
    fn read_file(file_location: &PathBuf) -> String
    {
        let mut file = match File::open(file_location)
        {
            Ok(i) => i,
            Err(err) =>
                {
                    eprintln!("Failed to open file {:?}. Additional info: {}", file_location, err.to_string());
                    exit(-1);
                }
        };

        let mut file_contents = String::new();

        if let Err(err) = file.read_to_string(&mut file_contents)
        {
            eprintln!("Failed to read file {:?}. Additional info: {}", file_location, err.to_string());
            exit(-1);
        }

        file_contents
    }
}

impl Drop for Shader
{
    fn drop(&mut self)
    {
       unsafe
           {
               gl::DeleteShader(self.shader);
           }
    }
}

impl Drop for ShaderProgram
{
    fn drop(&mut self)
    {
        unsafe
            {
                gl::DeleteProgram(self.shader_program);
            }
    }
}
