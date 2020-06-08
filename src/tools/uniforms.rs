use crate::tools::camera::Camera;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Uniforms {
    view_proj: na::Matrix4<f32>,
}

impl Uniforms {
    pub fn new() -> Self {
        let mut matrix: na::Matrix4<f32> = na::Matrix4::default();
        matrix.fill_with_identity();
        Self {
            view_proj: matrix,
        }
    }

    pub fn update_view_proj(&mut self, view_proj: na::Matrix4<f32>) {
        self.view_proj = view_proj;
    }
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}