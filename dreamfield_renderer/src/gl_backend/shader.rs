use std::ptr;
use std::ffi::CString;
use gl::types::*;
use super::bindings;
use strum::IntoEnumIterator;
use crate::resources::ShaderSource;

/// A shader program
pub struct ShaderProgram {
  id: u32
}

impl ShaderProgram {
    /// Build a shader program from the provided shader sources
    pub fn build(sources: &ShaderSource) -> Option<Self> {
        // Build shaders
        let shaders = match sources {
            ShaderSource::VertexFragment(vs, fs) => vec![
                ShaderProgram::compile_shader(gl::VERTEX_SHADER, &vs),
                ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, &fs)
            ],
            ShaderSource::VertexTessFragment(vs, tcs, tes, fs) => vec![
                ShaderProgram::compile_shader(gl::VERTEX_SHADER, &vs),
                ShaderProgram::compile_shader(gl::TESS_CONTROL_SHADER, &tcs),
                ShaderProgram::compile_shader(gl::TESS_EVALUATION_SHADER, &tes),
                ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, &fs)
            ]
        };

        // Link shader program
        let program_id = ShaderProgram::link_program(&shaders);

        // Delete shaders, they are no longer needed once the program is linked
        for shader in shaders {
            if let Some(shader) = shader {
                unsafe { gl::DeleteShader(shader) }
            }
        }

        // Create ShaderProgram struct
        program_id.map(|id| {
            // Create ShaderProgram instance
            let program = ShaderProgram { id };

            // Set standard uniform block bindings
            program.set_standard_uniform_block_bindings();

            program
        })
    }

    /// Get the gl id of the shader
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get a uniform location
    pub fn get_loc(&self, uniform_name: &str) -> i32 {
        let c_str = CString::new(uniform_name).unwrap();
        unsafe { gl::GetUniformLocation(self.id, c_str.as_ptr()) }
    }

    /// Set all the standard uniform block bindings
    fn set_standard_uniform_block_bindings(&self) {
        for binding in bindings::UniformBlockBinding::iter() {
            self.set_uniform_block_binding(&binding.to_string(), binding as u32);
        }
    }

    /// Set the binding for a uniform block
    fn set_uniform_block_binding(&self, uniform_block_name: &str, binding: u32) {
        let c_str = CString::new(uniform_block_name).unwrap();
        unsafe {
            let uniform_block_index = gl::GetUniformBlockIndex(self.id, c_str.as_ptr());
            if uniform_block_index != gl::INVALID_INDEX {
                gl::UniformBlockBinding(self.id, uniform_block_index, binding);
            }
        }
    }

    /// Bind the shader program
    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.id) };
    }

    /// Compile a shader
    fn compile_shader(shader_type: u32, shader_source: &str) -> Option<u32> {
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

                let shader_type_str = match shader_type {
                    gl::VERTEX_SHADER => "Vertex",
                    gl::FRAGMENT_SHADER => "Fragment",
                    _ => "<unknown>"
                };

                log::error!("{shader_type_str} shader failed to compile:\n");

                for (i, line) in shader_source.lines().enumerate() {
                    log::error!("[{i}] {line}");
                }

                log::error!("\n{message}\n");

                gl::DeleteShader(shader);

                None
            }
            else {
                Some(shader)
            }
        }
    }

    /// Link a shader program, deletes the passed in shaders automatically
    fn link_program(shaders: &[Option<u32>]) -> Option<u32> {
        unsafe {
            // Create shader program
            let shader_program = gl::CreateProgram();

            // Attach each shader
            for shader in shaders.iter() {
                if let Some(shader) = shader {
                    gl::AttachShader(shader_program, *shader);
                }
                else {
                    gl::DeleteProgram(shader_program);
                    return None;
                }
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

                log::error!("Shader program failed to link\n{}", std::str::from_utf8(&info_log).unwrap());

                gl::DeleteProgram(shader_program);

                None
            }
            else {
                Some(shader_program)
            }
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        log::debug!("Deleting shader program");
        unsafe { gl::DeleteProgram(self.id) }
    }
}
