use winit_input_helper::WinitInputHelper;
use winit::event::VirtualKeyCode as vkc;

use uv::projection::rh_ydown::perspective_vk;
use uv::mat::Mat4;
use uv::vec::Vec3;
use uv::rotor::Rotor3;
use uv::Isometry3;

pub struct Camera {
    perspective: Mat4,
    transformation: Isometry3,
}

impl Camera {
    pub fn new(sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let vertical_fov: f32 = std::f32::consts::PI / 3.0;
        let aspect = sc_desc.width as f32 / sc_desc.height as f32;
        let z_near: f32 = 0.1;
        let z_far: f32 = 100.0;

        let perspective: Mat4 = perspective_vk(vertical_fov, aspect, z_near, z_far);
        let mut transformation: Isometry3 = Isometry3::identity();
        transformation.translation.z -= 2.0;

        Self {
            perspective,
            transformation
        }
    }

    pub fn update_transformation(&mut self, transformation: Isometry3) {
        self.transformation = transformation;
    }

    fn transformation_matrix(&self) -> Mat4 {
        self.transformation.into_homogeneous_matrix()
    }

    pub fn to_matrix(&self) -> Mat4 {
        self.perspective  * self.transformation_matrix()
    }
}

pub struct CameraController {
    speed: f32,
    is_w: bool,
    is_s: bool,
    is_a: bool,
    is_d: bool,
    is_con: bool,
    is_space: bool,
    is_shift: bool,
    is_q: bool,
    is_e: bool,
    old_mouse_coords: (f32, f32),
    mouse_coords: (f32, f32),
}

impl CameraController {
    pub fn input(&mut self, input: &WinitInputHelper) {
        self.is_w = input.key_held(vkc::W);
        self.is_s = input.key_held(vkc::S);
        self.is_a = input.key_held(vkc::A);
        self.is_d = input.key_held(vkc::D);
        self.is_space = input.key_held(vkc::Space);
        self.is_shift = input.key_held(vkc::LShift);
        self.is_con = input.key_held(vkc::LControl);
        self.mouse_coords = match input.mouse() {
            Some(coords) => {
                (coords.0, coords.1)
            }
            _ => {
                (0.0, 0.0)
            }
        }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        match self.is_con {
            true => {
            }
            false => { }
        }

        match self.is_w {
            true => {

            }
            false => {}
        }
        match self.is_s {
            true => {

            }
            false => {}
        }
        match self.is_a {
            true => {

            }
            false => {}
        }
        match self.is_d {
            true => {

            }
            false => {}
        }
        match self.is_space {
            true => {

            }
            false => {}
        }
        match self.is_shift {
            true => {

            }
            false => {}
        }
        match self.is_q {
            true => {

            }
            false => {},
        }
        match self.is_e {
            true => {

            }
            false => {},
        }

        // XZ = YAW
        // YZ = PITCH
        // XY = ROLL
        let mouse_delta = (self.mouse_coords.0 - self.old_mouse_coords.0, self.mouse_coords.1 - self.old_mouse_coords.1);
        self.old_mouse_coords = self.mouse_coords.clone();
        let rotation: Rotor3 = Rotor3::from_rotation_yz(0.02) * Rotor3::from_rotation_xy(0.02);

        println!("{:?}", camera.transformation.rotation);
        camera.transformation.rotation = camera.transformation.rotation * rotation;
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 0.05,
            is_w: false,
            is_s: false,
            is_a: false,
            is_d: false,
            is_con: false,
            is_space: false,
            is_shift: false,
            is_q: false,
            is_e: false,
            old_mouse_coords: (0.0, 0.0),
            mouse_coords: (0.0, 0.0),
        }
    }
}