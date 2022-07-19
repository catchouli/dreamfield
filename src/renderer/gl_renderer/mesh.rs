use std::ptr;
use gl::types::*;

/// A mesh
pub struct Mesh {
    vao: u32,
    vbo: u32,
    ebo: Option<u32>
}

/// A vertex attribute
pub struct VertexAttrib {
    pub index: u32,
    pub attrib_type: u32,
    pub size: i32
}

impl Mesh {
    /// Create a new Mesh from a vertex buffer (non-indexed)
    pub fn new_basic(vertex_buffer: &[f32], buffer_layout: &[VertexAttrib]) -> Mesh {
        // Create and bind a vao
        let vao = Mesh::create_vao();

        // Create and bind buffers
        let vbo = Mesh::create_vbo(vertex_buffer);

        // Set up buffer layout
        Mesh::set_buffer_layout(buffer_layout);

        Mesh { vao, vbo, ebo: None }
    }

    /// Create a new Mesh from a vertex buffer and an index buffer
    pub fn new_indexed(vertex_buffer: &[f32], index_buffer: &[i32], buffer_layout: &[VertexAttrib]) -> Mesh {
        // Create and bind a vao
        let vao = Mesh::create_vao();

        // Create and bind buffers
        let vbo = Mesh::create_vbo(vertex_buffer);
        let ebo = Mesh::create_ebo(index_buffer);

        // Set up buffer layout
        Mesh::set_buffer_layout(buffer_layout);

        Mesh { vao, vbo, ebo: Some(ebo) }
    }

    /// Draw the mesh non-indexed
    pub fn draw_arrays(&self, element_type: u32, first: i32, count: i32) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(element_type, first, count);
        }
    }

    /// Draw the mesh indexed
    pub fn draw_indexed(&self, element_type: u32, element_count: i32) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(element_type, element_count, gl::UNSIGNED_INT, ptr::null());
        }
    }

    /// Create a vbo from a &[f32]
    fn create_vbo(vertex_buffer: &[f32]) -> u32 {
        unsafe {
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertex_buffer.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                           &vertex_buffer[0] as *const f32 as *const GLvoid,
                           gl::STATIC_DRAW);
            vbo
        }
    }

    /// Create an ebo from a &[i32]
    fn create_ebo(element_buffer: &[i32]) -> u32 {
        unsafe {
            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (element_buffer.len() * std::mem::size_of::<i32>()) as GLsizeiptr,
                           &element_buffer[0] as *const i32 as *const GLvoid,
                           gl::STATIC_DRAW);
            ebo
        }
    }

    /// Create a vao with a specified buffer layout
    fn create_vao() -> u32 {
        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);

            gl::BindVertexArray(vao);

            vao
        }
    }

    /// Set the buffer layout
    fn set_buffer_layout(vertex_attribs: &[VertexAttrib]) {
        unsafe {
            // Calculate stride
            let total_size = vertex_attribs.iter().fold(0, |total, attrib| total + attrib.size);
            let stride = total_size * std::mem::size_of::<f32>() as i32;

            // Bind and set up each attribute
            let mut offset: usize = 0;
            for vertex_attrib in vertex_attribs {
                gl::EnableVertexAttribArray(vertex_attrib.index);
                gl::VertexAttribPointer(vertex_attrib.index,
                                        vertex_attrib.size,
                                        vertex_attrib.attrib_type,
                                        gl::FALSE,
                                        stride,
                                        offset as *const GLvoid);

                offset += (vertex_attrib.size as usize) * std::mem::size_of::<f32>();
            }
        }
    }
}

impl Drop for Mesh {
    /// Clean up opengl buffers
    fn drop(&mut self) {
        unsafe {
            let ebo_str = self.ebo.map(|ebo| ebo.to_string()).unwrap_or("None".to_string());
            println!("Deleting mesh (vao={}, vbo={}, ebo={})", self.vao, self.vbo, ebo_str);

            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);

            if let Some(ebo) = &self.ebo {
                gl::DeleteBuffers(1, ebo);
            }
        }
    }
}
