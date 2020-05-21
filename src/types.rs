#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}
