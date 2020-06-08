use na::{
    geometry::UnitQuaternion,
    Vector3,
    Perspective3,
    Projective3,
    Translation3,
    Matrix4,
};

use winit_input_helper::WinitInputHelper;
use winit::event::VirtualKeyCode as vkc;


pub struct Camera {
    perspective: Perspective3<f32>,
    position: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
    aspect: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    pub fn new(sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let aspect = sc_desc.width as f32 / sc_desc.height as f32;
        let fov_y: f32 = std::f32::consts::PI / 3.0;
        let z_near: f32 = 0.1;
        let z_far: f32 = 100.0;
        let perspective = Perspective3::new(aspect, fov_y, z_near, z_far);
        let position = Vector3::new(0.0, 0.0, -2.0);
        let rotation = UnitQuaternion::identity();

        Self {
            perspective,
            position,
            rotation,
            aspect,
            fov_y,
            z_near,
            z_far
        }
    }

    // pub fn look_at(&mut self, target: Vector3<f32>) {
    //     self.view = self.view.as_matrix().look_at()
    // }

    fn projection_matrix(&self) -> Matrix4<f32> {
        let proj = self.perspective.clone();
        proj.into_inner()
    }

    fn view_matrix(&self) -> Matrix4<f32> {
        let rotation: Matrix4<f32> = self.rotation.into();
        let translation_matrix: Matrix4<f32>= Matrix4::new_translation(&self.position);
        rotation * translation_matrix
    }

    pub fn to_matrix(&self) -> Matrix4<f32> {
        let projection = self.projection_matrix();
        let view = self.view_matrix();
        projection * view
    }

    pub fn update_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }
}

pub struct CameraController {
    speed: f32,
    is_w: bool,
    is_s: bool,
    is_a: bool,
    is_d: bool,
    is_space: bool,
    is_shift: bool,
}

impl CameraController {
    pub fn input(&mut self, input: &WinitInputHelper) {
        self.is_w = input.key_held(vkc::W);
        self.is_s = input.key_held(vkc::S);
        self.is_a = input.key_held(vkc::A);
        self.is_d = input.key_held(vkc::D);
        self.is_space = input.key_held(vkc::Space);
        self.is_shift = input.key_held(vkc::LShift);
    }

    pub fn update(&self, camera: &mut Camera) {
        let mut new_position: Vector3<f32> = camera.position.clone();

        match self.is_w {
            true => new_position.data[2] += self.speed,
            false => {}
        }
        match self.is_s {
            true => new_position.data[2] -= self.speed,
            false => {}
        }
        match self.is_a {
            true => new_position.data[0] += self.speed,
            false => {}
        }
        match self.is_d {
            true => new_position.data[0] -= self.speed,
            false => {}
        }
        match self.is_space {
            true => new_position.data[1] -= self.speed,
            false => {}
        }
        match self.is_shift {
            true => new_position.data[1] += self.speed,
            false => {}
        }

        camera.update_position(new_position);
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 0.2,
            is_w: false,
            is_s: false,
            is_a: false,
            is_d: false,
            is_space: false,
            is_shift: false
        }
    }
}