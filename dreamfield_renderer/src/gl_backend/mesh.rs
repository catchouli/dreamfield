use std::ptr;
use gl::types::*;
use cgmath::{Vector2, Vector3, Vector4};

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
    pub fn new_indexed(vertex_buffer: &[f32], index_buffer: &[u32], buffer_layout: &[VertexAttrib]) -> Mesh {
        // Create and bind a vao
        let vao = Mesh::create_vao();

        // Create and bind buffers
        let vbo = Mesh::create_vbo(vertex_buffer);
        let ebo = Mesh::create_ebo(index_buffer);

        // Set up buffer layout
        Mesh::set_buffer_layout(buffer_layout);

        Mesh { vao, vbo, ebo: Some(ebo) }
    }

    /// Update the vbo
    pub fn update_vbo(&self, vertex_buffer: &[f32]) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertex_buffer.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                           &vertex_buffer[0] as *const f32 as *const GLvoid,
                           gl::DYNAMIC_DRAW);
        }
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

    /// Create an ebo from a &[u32]
    fn create_ebo(element_buffer: &[u32]) -> u32 {
        unsafe {
            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (element_buffer.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
                           &element_buffer[0] as *const u32 as *const GLvoid,
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
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            if let Some(ebo) = &self.ebo {
                gl::DeleteBuffers(1, ebo);
            }
        }
    }
}

/// An editable mesh, which stores the buffer as a vector in system memory so that it can be edited,
/// and re-uploads the buffer next time it's drawn. Currently only supports non-indexed meshes.
pub struct EditableMesh {
    mesh: Option<Mesh>,
    buffer_layout: Vec<VertexAttrib>,
    vertex_buffer: Vec<f32>,
    buffer_changed: bool,
}

impl EditableMesh {
    pub fn new(buffer_layout: Vec<VertexAttrib>) -> Self {
        Self {
            mesh: None,
            buffer_layout,
            vertex_buffer: Vec::new(),
            buffer_changed: true,
        }
    }

    /// Draw the mesh non-indexed
    pub fn draw_arrays(&mut self, element_type: u32, first: i32, count: i32) {
        if let Some(mesh) = &self.mesh {
            if self.buffer_changed {
                self.buffer_changed = false;
                mesh.update_vbo(&self.vertex_buffer);
            }
            mesh.draw_arrays(element_type, first, count);
        }
        else {
            self.buffer_changed = false;
            let mesh = Mesh::new_basic(&self.vertex_buffer, &self.buffer_layout);
            mesh.draw_arrays(element_type, first, count);
            self.mesh = Some(mesh);
        }
    }

    /// Clear the vertex buffer
    pub fn clear_vertex_buffer(&mut self) {
        self.buffer_changed = true;
        self.vertex_buffer.clear();
    }

    // Push an f32 into the vertex buffer
    pub fn push_f32(&mut self, v: f32) {
        self.buffer_changed = true;
        self.vertex_buffer.push(v);
    }

    // Push a vec2 into the vertex buffer
    pub fn push_vec2(&mut self, v: Vector2<f32>) {
        self.buffer_changed = true;
        self.vertex_buffer.push(v.x);
        self.vertex_buffer.push(v.y);
    }

    // Push a vec3 into the vertex buffer
    pub fn push_vec3(&mut self, v: Vector3<f32>) {
        self.buffer_changed = true;
        self.vertex_buffer.push(v.x);
        self.vertex_buffer.push(v.y);
        self.vertex_buffer.push(v.z);
    }

    // Push a vec4 into the vertex buffer
    pub fn push_vec4(&mut self, v: Vector4<f32>) {
        self.buffer_changed = true;
        self.vertex_buffer.push(v.x);
        self.vertex_buffer.push(v.y);
        self.vertex_buffer.push(v.z);
        self.vertex_buffer.push(v.w);
    }
}
