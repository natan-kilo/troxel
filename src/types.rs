#![allow(dead_code, unused_imports, unused_variables)]
use std::fmt;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (mem::size_of::<[f32; 4]>() + mem::size_of::<[f32; 3]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
/// A vertex with color instead of texture
pub struct VertexC {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl VertexC {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<VertexC>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for VertexC {}
unsafe impl bytemuck::Zeroable for VertexC {}

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}

impl Triangle {
    pub fn new(data: [Vertex; 3]) -> Self {
        Self {
            vertices: data.clone(),
        }
    }

    pub fn from_slice(bytes: &[Vertex; 3]) -> Self {
        Self { vertices: *bytes }
    }

    pub fn data(&self) -> &[Vertex; 3] {
        &self.vertices
    }

    pub fn change_vertex(&mut self, vertex_num: usize, new_vertex: &Vertex) {
        self.vertices[vertex_num] = *new_vertex;
    }

    pub fn set_data(&mut self, data: [Vertex; 3]) {
        self.vertices = data;
    }

    pub fn size(&self) -> usize {
        self.vertices.len()
    }

    pub fn as_ptr(&self) -> &Self {
        self
    }
}

unsafe impl bytemuck::Pod for Triangle {}
unsafe impl bytemuck::Zeroable for Triangle {}

// DATA ASSUMES:
// VERTICES     TRIANGLE VERTICES
// 0 -> 1       0.0 -> 0.1 / 1.0
// 2 -> 3       0.2 / 1.1 -> 1.2
//
// TODO FIX INDICES
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub vertices: [Vertex; 4],
    pub indices: [u16; 2],
}

impl Rectangle {
    pub fn new(data: &[Vertex; 4]) -> Self {
        let vertices = *data;
        let indices = [2, 3];
        Self { vertices, indices }
    }

    pub fn from_triangles(data: &[Triangle; 2]) -> Self {
        let triangles = *data;
        let vertices_tri0 = triangles[0].vertices;
        let vertices_tri1 = triangles[1].vertices;
        let vertices = [
            vertices_tri0[0],
            vertices_tri0[1],
            vertices_tri0[2],
            vertices_tri0[2],
        ];
        let indices = [2, 3];
        Self { vertices, indices }
    }
}

unsafe impl bytemuck::Pod for Rectangle {}
unsafe impl bytemuck::Zeroable for Rectangle {}
