use std::ptr;
use std::fs;
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
    pub fn new_from_vf(path: &str) -> ShaderProgram {
        // Load in raw shader source
        let raw_source = fs::read_to_string(path).unwrap();

        // Append the define for building vertex or fragment shader
        const VS_HEADER: &str = "#define BUILDING_VERTEX_SHADER";
        const FS_HEADER: &str = "#define BUILDING_FRAGMENT_SHADER";

        let vert_source = Self::append_after_version(&raw_source, VS_HEADER);
        let frag_source = Self::append_after_version(&raw_source, FS_HEADER);

        // Build shaders
        let vertex_shader = ShaderProgram::compile_shader(gl::VERTEX_SHADER, &vert_source);
        let fragment_shader = ShaderProgram::compile_shader(gl::FRAGMENT_SHADER, &frag_source);

        // Link shader program
        let shader_program = ShaderProgram::link_program(&vec![vertex_shader, fragment_shader]);

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

                println!("Fragment shader failed to compile:\n{message}\n\nShader source:\n");

                for (i, line) in shader_source.lines().enumerate() {
                    println!("[{i}] {line}");
                }
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

    /// Append the given string to the shader source, after the #version directive
    fn append_after_version(source: &str, append: &str) -> String {
        let mut updated_source = String::new();
        let mut version_directive_found = false;

        for line in source.lines() {
            updated_source.push_str(line);
            updated_source.push('\n');

            if !version_directive_found && line.contains("#version") {
                version_directive_found = true;
                updated_source.push_str(append);
                updated_source.push('\n');
            }
        }

        updated_source
    }
}

