use std::ptr;
use std::ffi::CString;
use gl::types::*;

/// A shader program
pub struct ShaderProgram {
  id: u32
}

impl ShaderProgram {
    /// Create a new shader program from a vertex and fragment shader source
    pub fn new_from_vf(vert: &str, frag: &str) -> ShaderProgram {
        let vertex_shader = ShaderProgram::compile_shader(gl::VERTEX_SHADER, vert);
        let fragment_shader = ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, frag);

        let shaders = vec![vertex_shader, fragment_shader];

        let shader_program = ShaderProgram::link_program(&shaders);

        ShaderProgram { id: shader_program }
    }

    /// Get the gl id of the shader
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Bind the shader program
    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.id) };
    }

    /// Compile a shader
    fn compile_shader(shader_type: u32, shader_source: &str) -> u32 {
        unsafe {
            // Create new shader
            let shader = gl::CreateShader(shader_type);

            // Set shader source
            let shader_source_cstr = CString::new(shader_source.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &shader_source_cstr.as_ptr(), ptr::null());

            // Compile shader
            gl::CompileShader(shader);

            // Check for errors, we intentionally return like normal even if it errors
            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                const MAX_LOG_SIZE: usize = 512;

                let mut length: i32 = 0;
                let mut info_log = vec![0; MAX_LOG_SIZE];

                gl::GetShaderInfoLog(shader, MAX_LOG_SIZE as i32, &mut length as *mut i32,
                                     info_log.as_mut_ptr() as *mut GLchar);

                info_log.set_len(length as usize);
                let message = std::str::from_utf8(&info_log).unwrap();

                println!("Fragment shader failed to compile:\n{message}");
            }

            shader
        }
    }

    /// Link a shader program, deletes the passed in shaders automatically
    fn link_program(shaders: &[u32]) -> u32 {
        unsafe {
            // Create shader program
            let shader_program = gl::CreateProgram();

            // Attach each shader
            for &shader in shaders {
                gl::AttachShader(shader_program, shader);
            }

            // Link the program
            gl::LinkProgram(shader_program);

            // Check for errors
            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                const MAX_LOG_SIZE: usize = 512;

                let mut length: i32 = 0;
                let mut info_log = vec![0; MAX_LOG_SIZE];

                gl::GetProgramInfoLog(shader_program, MAX_LOG_SIZE as i32, &mut length as *mut i32,
                    info_log.as_mut_ptr() as *mut GLchar);

                info_log.set_len(length as usize);

                println!("Shader program failed to link\n{}", std::str::from_utf8(&info_log).unwrap());
            }

            // Delete shaders
            for shader in shaders {
                gl::DeleteShader(*shader);
            }

            shader_program
        }
    }
}

