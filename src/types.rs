#![allow(dead_code, unused_imports, unused_variables)]
use std::fmt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Triangle {
    vertecies: [Vertex; 3],
}

impl Triangle {
    pub fn new(data: [Vertex; 3]) -> Self {
        Self {
            vertecies: data.clone(),
        }
    }

    pub fn from_slice(bytes: &[Vertex; 3]) -> Self {
        Self { vertecies: *bytes }
    }

    pub fn data(&self) -> &[Vertex; 3] {
        &self.vertecies
    }

    pub fn change_vertex(&mut self, vertex_num: usize, new_vertex: &Vertex) {
        self.vertecies[vertex_num] = *new_vertex;
    }

    pub fn set_data(&mut self, data: [Vertex; 3]) {
        self.vertecies = data;
    }

    pub fn size(&self) -> usize {
        self.vertecies.len()
    }

    pub fn as_ptr(&self) -> &Self {
        self
    }
}

impl fmt::Pointer for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // use `as` to convert to a `*const T`, which implements Pointer, which we can use

        let ptr = self as *const Self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

pub struct Rectangle {
    pub vertecies: [Vertex; 4],
}

impl Rectangle {
    pub fn new(data: &[Vertex; 4]) -> Self {
        Self {
            vertecies: data.clone(),
        }
    }

    pub fn new_rect(point_a: Vertex, length: f32) -> Self {
        unimplemented!();
    }
}
