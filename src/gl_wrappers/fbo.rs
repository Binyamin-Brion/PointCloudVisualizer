use nalgebra_glm::{TMat4, TVec4, vec4};
use crate::rendering::camera::{Camera, CameraType};

/// Represents a frame buffer object, providing an abstraction
///  over its opreations
pub struct FBO
{
    fbo: u32,
    texture: u32,
    texture_dimensions: (i32, i32),
    binding_point: u32,
    camera: Camera
}

/// Represents the type of textures that the frame buffer object
/// can write to
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TextureType
{
    RGB8 = gl::RGBA8,
    DepthComponent = gl::DEPTH_COMPONENT24,
}

impl FBO
{
    /// Creates a new frame buffer object capable of writing to a texture of the type
    /// provided (both type and dimensions)
    ///
    /// `texture_dimensions` - the dimensions of the texture the FBO will write to
    /// `binding_point` - the binding point of the sampler that the FBO's texture will bind to
    /// `camera_type` - the type of camera the FBO will use to render a scene into its texture
    /// `texture_type` - the format of the texture the FBO will write to
    pub fn new(texture_dimensions: (i32, i32), binding_point: u32, camera_type: CameraType, texture_type: TextureType) -> Result<FBO, ()>
    {
        let mut fbo: u32 = 0;
        let mut texture: u32 = 0;

        unsafe
            {
                gl::CreateFramebuffers(1, &mut fbo);
                gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);

                // Create texture the FBO will write to
                gl::TextureStorage2D(texture, 1, texture_type as u32, texture_dimensions.0 as i32, texture_dimensions.1 as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);

                // Not strictly needed, but does provided a known default
                let border_colour: TVec4<f32> = vec4(1.0, 1.0, 1.0, 1.0);
                gl::TextureParameterfv(texture, gl::TEXTURE_BORDER_COLOR, border_colour.as_ptr());

                match texture_type
                {
                    TextureType::RGB8 => gl::NamedFramebufferTexture(fbo, gl::COLOR_ATTACHMENT0, texture, 0),
                    TextureType::DepthComponent =>
                        {
                            gl::NamedFramebufferTexture(fbo, gl::DEPTH_ATTACHMENT, texture, 0);
                            gl::NamedFramebufferDrawBuffer(fbo, gl::NONE);
                            gl::NamedFramebufferReadBuffer(fbo, gl::NONE);
                        },
                }

                if gl::CheckNamedFramebufferStatus(fbo, gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE
                {
                    eprintln!("Failed to create FBO!");
                    return Err(());
                }
            }

        let camera = Camera::new(camera_type);

        Ok(FBO { fbo, texture, texture_dimensions, binding_point, camera })
    }

    /// Get a mutable reference to the camera the FBO uses for rendering
    pub fn get_mut_camera(&mut self) -> &mut Camera
    {
        &mut self.camera
    }

    /// Get a reference to the camera the FBO uses for rendering
    pub fn get_camera(&self) -> &Camera
    {
        &self.camera
    }

    /// Get the projection view matrix of the FBO's camera
    pub fn get_projection_view_matrix(&self) -> TMat4<f32>
    {
        // This is a helper function; same result can be obtained by
        // getting reference to the FBO's camera
        self.camera.get_projection_view_matrix()
    }

    /// Prepares the FBO for subsequent draw calls that will write into
    /// the FBO's texture
    pub fn bind_for_drawing(&self)
    {
        unsafe
            {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
                gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::Viewport(0, 0, self.texture_dimensions.0, self.texture_dimensions.1);
            }
    }

    /// Binds the FBO's texture to the sampler binding point provided
    /// in the constructor
    pub fn bind_draw_result(&self)
    {
        unsafe
            {
                gl::BindTextureUnit(self.binding_point, self.texture);
            }
    }
}

impl Drop for FBO
{
    fn drop(&mut self)
    {
        unsafe
            {
                gl::DeleteFramebuffers(1, &self.fbo);
            }
    }
}
