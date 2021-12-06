use std::ffi::{c_void, CStr};
use std::process::exit;
use std::ptr::null;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Glfw, Key, MouseButton, Window, WindowEvent, WindowHint};

/// Abstraction of the window that is rendered to
pub struct RenderWindow
{
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
    key_input: Vec<(Key, Action)>,
    cursor_pos_history: Vec<(i32, i32)>,
    cursor_button_history: Vec<(MouseButton, Action)>,
    latest_cursor_pos: (i32, i32),
}

impl RenderWindow
{
    /// Creates a new window, and after this function all OpenGL functions can be called
    ///
    /// `window_size` - the dimensions of the window (width, height)
    /// `window_title` - the name of the window
    /// `window_hints`- additional information about how the window should behave or initialize the
    ///                 OpenGL context. If the window hints contain a DebugContext request, then the
    ///                 context will be in debug mode and all warnings printed to the console
    pub fn new(window_size: (u32, u32), window_title: String, window_hints: Vec<WindowHint>) -> RenderWindow
    {
        let debug_mode =
            {
                let mut use_debug_context = false;
                for x in &window_hints
                {
                    match x
                    {
                        glfw::WindowHint::OpenGlDebugContext(true) =>
                            {
                                use_debug_context = true;
                                break;
                            },
                        _ => {}
                    }
                }
                use_debug_context
            };

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        for x in window_hints
        {
            glfw.window_hint(x);
        }

        let (mut window, events) = match glfw.create_window
        (
            window_size.0,
            window_size.1,
            window_title.as_str(),
            glfw::WindowMode::Windowed
        )
        {
            Some(i) => i,
            None =>
                {
                    eprintln!("Failed to create the render window");
                    exit(-1);
                }
        };

        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_size_polling(true);
        window.make_current();
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        unsafe
            {
                gl::Viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
            }

        if debug_mode
        {
            RenderWindow::setup_debug_context();
        }

        RenderWindow{ glfw, window, events, key_input: Vec::new(), cursor_pos_history: Vec::new(), cursor_button_history: Vec::new(), latest_cursor_pos: (0, 0) }
    }

    /// Query if the window should be closed
    pub fn should_close(&self) -> bool
    {
        self.window.should_close()
    }

    /// Swap the offscreen buffer with the onscreen buffer
    pub fn swap_buffers(&mut self)
    {
        self.window.swap_buffers();
    }

    /// Get all of the input keys for the current frame
    pub fn get_key_input(&self) -> &Vec<(Key, Action)>
    {
        &self.key_input
    }

    /// Get all of the cursor history for the current frame
    pub fn get_cursor_history(&self) -> &Vec<(i32, i32)>
    {
        &self.cursor_pos_history
    }

    /// Get the cursor button history for the current frame
    pub fn get_cursor_button_history(&self) -> &Vec<(MouseButton, Action)>
    {
        &self.cursor_button_history
    }

    pub fn get_window_dimensions(&self) -> (i32, i32)
    {
        self.window.get_size()
    }

    /// Tell the window that it should close
    pub fn set_window_should_close(&mut self, close: bool)
    {
        self.window.set_should_close(close);
    }

    pub fn get_latest_cursor_pos(&self) -> (i32, i32)
    {
        self.latest_cursor_pos
    }

    /// Find all events that have occurred for the current frame
    pub fn poll_events(&mut self)
    {
        self.glfw.poll_events();
        self.key_input.clear();
        self.cursor_pos_history.clear();
        self.cursor_button_history.clear();

        for (_, event) in glfw::flush_messages(&self.events)
        {
            match event
            {
                glfw::WindowEvent::Key(key, _, action, _) =>
                {
                    self.key_input.push((key, action));
                },
                glfw::WindowEvent::Size(width, height) =>
                    {
                        unsafe
                            {
                                println!("Resized to: {}, {}", width, height);
                                gl::Viewport(0, 0, width, height);
                            }
                    },
                glfw::WindowEvent::CursorPos(x, y) =>
                    {
                        self.cursor_pos_history.push((x as i32, y as i32));
                        self.latest_cursor_pos = (x as i32, y as i32);
                    },
                glfw::WindowEvent::MouseButton(button, action, _) =>
                    {
                        self.cursor_button_history.push((button, action))
                    }
                _ => {}
            }
        }
    }

    /// Configures the OpenGL context for debugging
    fn setup_debug_context()
    {
        unsafe
            {
                let mut flags = 0;
                gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags);
                if flags as u32 & gl::CONTEXT_FLAG_DEBUG_BIT != 0
                {
                    gl::Enable(gl::DEBUG_OUTPUT);
                    gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS); // makes sure errors are displayed synchronously
                    gl::DebugMessageCallback(Some(gl_debug_output), null());
                    gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, null(), gl::TRUE);
                }
                else
                {
                    eprintln!("Debug Context not active! Check if your driver supports the extension.")
                }
            }
    }
}

/// Debugging function that prints warnings related to OpenGL state and function calls
extern "system" fn gl_debug_output(_: gl::types::GLenum,
                                   _: gl::types::GLenum,
                                   _: gl::types::GLuint,
                                   _: gl::types::GLenum,
                                   _length: gl::types::GLsizei,
                                   message: *const gl::types::GLchar,
                                   _user_param: *mut c_void)
{
    let message = unsafe { CStr::from_ptr(message).to_str().unwrap() };
    println!("{}", message);
}