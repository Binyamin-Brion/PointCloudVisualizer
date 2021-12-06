use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null;
use angel_font_file_parser::{AtlasDimensions, CharacterInfo};
use angel_font_file_parser::extract_characters;
use nalgebra_glm::{TMat4, TVec2, vec2};
use stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load;
use stb_image::image::LoadResult;
use crate::gl_wrappers::buffer::{Buffer, BufferType};
use crate::helper_logic::folder_location_functions::{get_shaders_folder, get_text_folder};
use crate::gl_wrappers::shader_program_creation::{ShaderInitInfo, ShaderProgram, ShaderType};
use crate::gl_wrappers::vao::VAO;

/// Logic and components required to render text
pub struct TextRendering
{
    texture: u32,
    shader_program: ShaderProgram,
    vao: VAO,
    plane_buffer: Buffer,
    tex_coords_buffer: Buffer,
    // This variable is kept to logically show that the VBO it is representing is kept alive for the
    // duration of the program. However, it is never modified after the the TextRendering constructor
    // has run. To silence a compiler warning, the underscore is used
    _indice_buffer: Buffer,
    char_info: Vec<CharacterInfo>,
    window_dimensions: (i32, i32),
    camera_matrix: TMat4<f32>,

    character_vertices: Vec<TVec2<f32>>,
    character_tex_coords: Vec<[(f32, f32); 4]>,
    num_characters: i32,
    sentence_positions: Vec<SentenceIndex>,

    default_window_width: f32,
    default_window_height: f32,
}

/// Represents a single line of text to buffer
struct SentenceIndex
{
    starting_index: i32, // Out of all the characters buffered
    starting_position: TVec2<f32>, // In pixels
}

/// Reduce the boilerplate to check if all information required to render a character is available
macro_rules! verify_char_info {
    ($variable: tt, $char_info: tt, $value: expr) =>
    {{
        let value = match $char_info.$variable
        {
            Some(i) => i as f32,
            None =>
                {
                    eprintln!("Char id {} does not have a $variable", $value);
                    continue;
                }
        };
        value
    }};
}

impl TextRendering
{
    /// Creates a new TextRendering structure capable of rendering text to a window of the given size
    ///
    /// `window_dimensions` - the dimensions of the window being rendered to
    pub fn new(window_dimensions: (i32, i32)) -> TextRendering
    {
        unsafe{ stbi_set_flip_vertically_on_load(1); }
        let texture_load = match stb_image::image::load(get_text_folder().join("robotoFont.png"))
        {
            LoadResult::Error(_) | LoadResult::ImageF32(_) => panic!("Could not file: {:?}", get_text_folder().join("robotoFont.png")),
            LoadResult::ImageU8(i) => i
        };

        let mut texture: u32 = 0;

        unsafe
            {
                // Texture atlas of characters that can be rendered
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

                gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
                gl::TextureStorage2D(texture, 1, gl::RGBA8, texture_load.width as i32, texture_load.height as i32);

                gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
                gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);

                gl::TextureSubImage2D(texture, 0, 0, 0, texture_load.width as i32, texture_load.height as i32,
                                        gl::RGBA, gl::UNSIGNED_BYTE, texture_load.data.as_ptr() as *const c_void);
            }

        let vao = VAO::new();
        vao.bind_vao();
        // These correspond to "textVertexShader.glsl" in the shaders folder
        vao.specify_index_layout(0, 2, gl::FLOAT, false, 0);
        vao.specify_index_layout(1, 2, gl::FLOAT, false, 0);

        // More than enough as of time of writing
        let max_number_characters = 1000;

        let plane_buffer = Buffer::new(&vao, max_number_characters * (size_of::<TVec2<f32>>() * 4) as isize, 3, BufferType::Array(0, 8));
        let tex_coords_buffer = Buffer::new(&vao, max_number_characters * (size_of::<TVec2<f32>>() * 4) as isize, 3, BufferType::Array(1, 8));
        let mut indice_buffer = Buffer::new(&vao, (size_of::<u32>() * 6) as isize, 1, BufferType::Indice);

        // Indices to render a rectangle. Vertices to render a character rectangle are done later
        indice_buffer.write_data(&vec![0_u32, 1, 2, 2, 0, 3], &vao, 5_000_000);

        let shader_program = ShaderProgram::new
            (
                vec!
                [
                    ShaderInitInfo{ shader_type: ShaderType::Vertex, shader_location: get_shaders_folder().join("textVertexShader.glsl") },
                    ShaderInitInfo{ shader_type: ShaderType::Fragment, shader_location: get_shaders_folder().join("textFragmentShader.glsl") },
                ]
            );

        let char_info = extract_characters
            (get_text_folder().join("robotoFont.fnt"),
            AtlasDimensions{ width: texture_load.width as i32, height: texture_load.height as i32 }
            ).unwrap();

        TextRendering
        {
            _indice_buffer: indice_buffer,
            texture,
            shader_program,
            vao,
            plane_buffer,
            tex_coords_buffer,
            char_info,
            window_dimensions,
            // The location of the characters are specified in pixels due to this
            camera_matrix: nalgebra_glm::ortho(0.0, window_dimensions.0 as f32, 0.0, window_dimensions.1 as f32, 0.0, 1.0),
            character_vertices: vec![],
            character_tex_coords: vec![],
            num_characters: 0,
            sentence_positions: vec![],
            default_window_width: 1280.0,
            default_window_height: 720.0
        }
    }

    /// Notifies the text renderer that the window being rendered to has changed its dimensions
    ///
    /// `window_dimensions` - the new dimensions of the render window
    pub fn update_window_dimensions(&mut self, window_dimensions: (i32, i32))
    {
        self.window_dimensions = window_dimensions;
        self.camera_matrix = nalgebra_glm::ortho(0.0, window_dimensions.0 as f32, 0.0, window_dimensions.1 as f32, 0.0, 1.0);
    }

    /// Prepares the required rendering information to render the given text
    ///
    /// `text` - the text to render
    /// `starting_position` - the position to start rendering the text, specified as Normalized Device Coordinates
    ///                       with the viewport of the most recent window dimensions given to the text renderer
    /// `max_num_char` - maximum number of char of the provided text to render. Any excess characters are not rendered
    pub fn buffer_text_for_rendering<A: AsRef<str>>(&mut self, text: A, mut starting_position: TVec2<f32>, max_num_char: usize)
    {
        // Convert the starting position from NDC to pixels
        starting_position.x *= self.window_dimensions.0 as f32;
        starting_position.y *= self.window_dimensions.1 as f32;
        self.sentence_positions.push(SentenceIndex{starting_index: self.num_characters, starting_position});

        // This is relative to the starting point
        let mut total_offset_x = 0.0_f32;

        for (index, c) in text.as_ref().chars().filter(|c| (*c as usize) < 128).enumerate()
        {
            // Two characters- null and line feed- need special cases to index into the character info array
            let char_info = match c as usize
            {
                0 => &self.char_info[0],
                10 => &self.char_info[1],
                _ => &self.char_info[c as usize - 30]
            };

            let char_width = verify_char_info!(width, char_info, c as usize);
            let char_height = verify_char_info!(height, char_info, c as usize);
            let char_x_offset = verify_char_info!(x_offset, char_info, c as usize);
            let char_yoffset = verify_char_info!(y_offset, char_info, c as usize);
            let char_x_advance = verify_char_info!(x_advance, char_info, c as usize);

            // Check that when loading the character information if no valid coordinates could be
            // found to get the texture information to render a character
            if TextRendering::verify_tex_coords(&char_info.texture_coordinates)
            {
                eprintln!("Invalid texture coordinates for char id {}", c as usize);
                continue;
            }

            if index > max_num_char
            {
                break;
            }

            if (c as usize) != 32
            {
                // This is for the current character being processed
                let local_offset_x = total_offset_x + char_x_offset;
                let local_offset_y = char_yoffset;

                // Specify the character plane that the character will be rendered to (in pixels)
                // In the order: top left, top right, bottom left, bottom right
                self.character_vertices.push(vec2(local_offset_x, local_offset_y));
                self.character_vertices.push(vec2(local_offset_x, local_offset_y + char_height));
                self.character_vertices.push(vec2(local_offset_x + char_width, local_offset_y  + char_height));
                self.character_vertices.push(vec2(local_offset_x + char_width, local_offset_y));

                self.character_tex_coords.push(char_info.texture_coordinates);

                self.num_characters += 1;
                // Only enough space reserved to render 1000 characters
                if self.num_characters >= 1000
                {
                    break;
                }
            }

            // Advance the virtual cursor
            total_offset_x += char_x_advance + 5.0;
        }
    }

    /// Render the buffered text
    pub fn render_buffered_text(&mut self)
    {
        // This is required to handle the last buffer character due to the loop logic below. No
        // character will be rendered
        self.buffer_text_for_rendering("", vec2(0.0, 0.0), 0);

        unsafe
            {
                gl::Disable(gl::DEPTH_TEST);
                gl::Viewport(0, 0, (self.window_dimensions.0 as f32 * 0.665 ) as i32, self.window_dimensions.1);
                gl::BindTextureUnit(0, self.texture);
            }

        self.plane_buffer.write_data(&self.character_vertices, &self.vao, 5_000_000);
        self.tex_coords_buffer.write_data(&self.character_tex_coords, &self.vao, 5_000_000);

        self.shader_program.use_program();
        self.vao.bind_vao();
        self.shader_program.write_mat4("projectionViewMatrix", &self.camera_matrix);
        self.shader_program.write_float("textScaleX", self.window_dimensions.0 as f32 / self.default_window_width);
        self.shader_program.write_float("textScaleY", self.window_dimensions.1 as f32 / self.default_window_height);

        for x in 0..self.sentence_positions.len() - 1
        {
            self.shader_program.write_vec2("translation", &self.sentence_positions[x].starting_position);

            // Number of characters in the current sentence
            let number_characters = self.sentence_positions[x + 1].starting_index - self.sentence_positions[x].starting_index;
            for i in 0..number_characters
            {
                let char_index = i + self.sentence_positions[x].starting_index;
                unsafe{ gl::DrawElementsBaseVertex(gl::TRIANGLES, 6, gl::UNSIGNED_INT, null(), (char_index * 4) as i32) }
            }
        }

        self.plane_buffer.update_fence();
        self.tex_coords_buffer.update_fence();

        self.num_characters = 0;
        self.sentence_positions.clear();
        self.character_vertices.clear();
        self.character_tex_coords.clear();

        unsafe{ gl::Enable(gl::DEPTH_TEST) }
    }

    /// Check that the given texture coordinates are valid (as in will result in a recognizable
    /// portion of the texture atlas being rendered to a quad)
    fn verify_tex_coords(tex_coords: &[(f32, f32); 4]) -> bool
    {
        tex_coords[0].0 == 0.0 && tex_coords[0].1 == 0.0 &&
        tex_coords[1].0 == 0.0 && tex_coords[1].1 == 0.0 &&
        tex_coords[2].0 == 0.0 && tex_coords[2].1 == 0.0 &&
        tex_coords[3].0 == 0.0 && tex_coords[3].1 == 0.0
    }
}