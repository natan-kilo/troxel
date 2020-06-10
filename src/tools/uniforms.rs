use crate::tools::camera::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniforms {
    view_proj: uv::Mat4,
}

impl Uniforms {
    pub fn new() -> Self {
        let mut matrix: uv::Mat4 = uv::Mat4::identity();
        Self {
            view_proj: matrix,
        }
    }

    pub fn update_view_proj(&mut self, view_proj: uv::Mat4) {
        self.view_proj = view_proj;
    }
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}