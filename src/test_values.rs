#![allow(dead_code)]

use crate::types::{Rectangle, Triangle, Vertex};

// VERTECIES
pub const TRIANGLE: [Vertex; 3] = [
    Vertex {
        pos: [-0.5, -0.5, 0.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, 0.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.0, 0.5, 0.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

pub const MESH: [Vertex; 6] = [
    Vertex {
        pos: [0.0, -1.0, 0.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [-1.0, 0.0, 0.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.0, -1.0, 0.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 0.0, 0.0],
        color: [1.0, 1.0, 0.0, 1.0],
    },
];
