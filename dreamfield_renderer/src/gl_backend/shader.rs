use std::ptr;
use std::ffi::CString;
use gl::types::*;
use super::bindings;
use strum::IntoEnumIterator;

/// A shader program
pub struct ShaderProgram {
  id: u32
}

impl ShaderProgram {
    /// Create a new shader program from a vertex and fragment shader source
    pub fn new_from_vf((vertex, fragment): (&str, &str)) -> ShaderProgram {
        // Build shaders
        let vertex_shader = ShaderProgram::compile_shader(gl::VERTEX_SHADER, &vertex);
        let fragment_shader = ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, &fragment);

        // Link shader program
        let shader_program = ShaderProgram::link_program(&vec![vertex_shader, fragment_shader]);

        // Create ShaderProgram instance
        let program = ShaderProgram { id: shader_program };

        // Set standard uniform block bindings
        program.set_standard_uniform_block_bindings();

        program
    }

    /// Create a new shader program from a vertex, tesselation, geometry and fragment shader source
    pub fn new_from_vtgf((vertex, tess_control, tess_eval, geometry, fragment): (&str, &str, &str, &str, &str))
        -> ShaderProgram
    {
        // Build shaders
        let vertex_shader = ShaderProgram::compile_shader(gl::VERTEX_SHADER, &vertex);
        let tess_control_shader = ShaderProgram::compile_shader(gl::TESS_CONTROL_SHADER, &tess_control);
        let tess_eval_shader = ShaderProgram::compile_shader(gl::TESS_EVALUATION_SHADER, &tess_eval);
        let geometry_shader = ShaderProgram::compile_shader(gl::GEOMETRY_SHADER, &geometry);
        let fragment_shader = ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, &fragment);

        // Link shader program
        let shader_program = ShaderProgram::link_program(&vec![vertex_shader, tess_control_shader, tess_eval_shader,
                                                         geometry_shader, fragment_shader]);

        // Create ShaderProgram instance
        let program = ShaderProgram { id: shader_program };

        // Set standard uniform block bindings
        program.set_standard_uniform_block_bindings();

        program
    }

    /// Create a new shader program from a vertex, tesselation and fragment shader source
    pub fn new_from_vtf((vertex, tess_control, tess_eval, fragment): (&str, &str, &str, &str)) -> ShaderProgram {
        // Build shaders
        let vertex_shader = ShaderProgram::compile_shader(gl::VERTEX_SHADER, &vertex);
        let tess_control_shader = ShaderProgram::compile_shader(gl::TESS_CONTROL_SHADER, &tess_control);
        let tess_eval_shader = ShaderProgram::compile_shader(gl::TESS_EVALUATION_SHADER, &tess_eval);
        let fragment_shader = ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, &fragment);

        // Link shader program
        let shader_program = ShaderProgram::link_program(&vec![vertex_shader, tess_control_shader, tess_eval_shader,
                                                         fragment_shader]);

        // Create ShaderProgram instance
        let program = ShaderProgram { id: shader_program };

        // Set standard uniform block bindings
        program.set_standard_uniform_block_bindings();

        program
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

                log::error!("Shader program failed to link\n{}", std::str::from_utf8(&info_log).unwrap());
            }

            // Delete shaders
            for shader in shaders {
                gl::DeleteShader(*shader);
            }

            shader_program
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        log::debug!("Deleting shader program");
        unsafe { gl::DeleteProgram(self.id) }
    }
}
