use std::{collections::HashMap, sync::Arc};
use crate::gl_backend::ShaderProgram;

/// Shader sources
pub enum ShaderSource {
    /// A shader program with a vertex and fragment shader source
    VertexFragment(&'static str, &'static str),
    /// A shader program with a vertex, tessellation evaluation, tessellation control, and fragment shader source
    VertexTessFragment(&'static str, &'static str, &'static str, &'static str)
}

// A single shader
pub struct ShaderEntry {
    compiled: bool,
    source: ShaderSource,
    program: Option<Arc<ShaderProgram>>
}

impl ShaderEntry {
    /// Get the shader program, compiling it if it isn't already compiled
    pub fn program(&mut self) -> Result<&Arc<ShaderProgram>, String> {
        if self.program.is_none() && !self.compiled {
            self.compiled = true;
            self.program = ShaderProgram::build(&self.source).map(Arc::new);
        }
        self.program.as_ref().ok_or("Failed to compile shader".to_string())
    }
}

// Shader manager
pub struct ShaderManager {
    entries: HashMap<String, ShaderEntry>
}

impl ShaderManager {
    pub fn new(sources: Vec<(&str, ShaderSource)>) -> Self {
        let entries = sources.into_iter().map(|(name, source)| {
            (name.to_string(), ShaderEntry { source, program: None, compiled: false })
        }).collect();

        Self {
            entries
        }
    }

    pub fn get(&mut self, name: &str) -> Result<&Arc<ShaderProgram>, String> {
        self.entries.get_mut(name)
            .map(|shader| shader.program())
            .unwrap_or(Err(format!("No such shader {}", name)))
    }
}
