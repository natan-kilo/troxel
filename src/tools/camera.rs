use winit_input_helper::WinitInputHelper;
use winit::event::VirtualKeyCode as vkc;

use uv::projection::rh_ydown::perspective_vk;
use uv::mat::Mat4;
use uv::vec::Vec3;
use uv::rotor::Rotor3;
use uv::Isometry3;
use crate::utils;
use crate::utils::{rotor_from_angles, rad};
use std::f32::consts::PI;

pub struct Camera {
    perspective: Mat4,
    transformation: Isometry3,
    aspect: f32,
    fov: f32,
}

impl Camera {
    pub fn new(sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let vertical_fov: f32 = utils::rad(60.0);
        let aspect = sc_desc.width as f32 / sc_desc.height as f32 ;
        let z_near: f32 = 0.1;
        let z_far: f32 = 100.0;

        let perspective: Mat4 = perspective_vk(vertical_fov, aspect, z_near, z_far);
        let mut transformation: Isometry3 = Isometry3::identity();
        transformation.translation.z -= 2.0;

        Self {
            perspective,
            transformation,
            aspect,
            fov: vertical_fov,
        }
    }

    pub fn update_transformation(&mut self, transformation: Isometry3) {
        self.transformation = transformation;
    }

    fn transformation_matrix(&self) -> Mat4 {
        self.transformation.into_homogeneous_matrix()
    }

    pub fn to_matrix(&self) -> Mat4 {
        self.perspective * self.transformation_matrix()
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
    zoom: f32,

    old_mouse_coords: (f32, f32),
    mouse_coords: (f32, f32),
}

impl CameraController {
    pub fn input(&mut self, input: &WinitInputHelper) {
        self.is_w = input.key_held(vkc::W);
        self.is_s = input.key_held(vkc::S);
        self.is_a = input.key_held(vkc::A);
        self.is_d = input.key_held(vkc::D);
        self.is_q = input.key_held(vkc::Q);
        self.is_e = input.key_held(vkc::E);
        self.is_space = input.key_held(vkc::Space);
        self.is_shift = input.key_held(vkc::LShift);
        self.is_con = input.key_held(vkc::LControl);

        self.mouse_coords = match input.mouse() {
            Some(input) => input,
            _ => (0.0, 0.0)
        };

        self.zoom += input.scroll_diff();
        if self.zoom >= 6.0 {
            self.zoom -= 1.0;
        }
        if self.zoom <= - 10.0 {
            self.zoom += 1.0
        }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        self.update_rotation(camera);

        let forward: Vec3 = Vec3::new(0.0, 0.0, 1.0);
        let left: Vec3 = Vec3::new(1.0, 0.0, 0.0);
        let up: Vec3 = Vec3::new(0.0, -1.0, 0.0);

        match self.is_con {
            true => {
            }
            false => { }
        }

        match self.is_w {
            true => {
                camera.transformation.translation += forward * self.speed;
            }
            false => {}
        }
        match self.is_s {
            true => {
                camera.transformation.translation -= forward * self.speed;
            }
            false => {}
        }
        match self.is_a {
            true => {
                camera.transformation.translation += left * self.speed;
            }
            false => {}
        }
        match self.is_d {
            true => {
                camera.transformation.translation -= left * self.speed;
            }
            false => {}
        }
        match self.is_space {
            true => {
                camera.transformation.translation += up * self.speed;
            }
            false => {}
        }
        match self.is_shift {
            true => {
                camera.transformation.translation -= up * self.speed;
            }
            false => {}
        }
        match self.is_q {
            true => {
                camera.transformation.append_rotation(rotor_from_angles(0.0, 0.0, -1.0));
            }
            false => {},
        }
        match self.is_e {
            true => {
                camera.transformation.append_rotation(rotor_from_angles(0.0, 0.0, 1.0));
            }
            false => {},
        }

        self.change_fov(camera);
    }

    /// >:[
    fn update_rotation(&mut self, camera: &mut Camera) {
        if self.mouse_coords.0 == self.old_mouse_coords.0 && self.mouse_coords.1 == self.old_mouse_coords.1 {
            return;
        }

        let dx: f32 = -self.mouse_coords.0 + self.old_mouse_coords.0;
        let dy: f32 = self.mouse_coords.1 - self.old_mouse_coords.1;

        self.old_mouse_coords = self.mouse_coords;

        let offset_rotor: Rotor3 = utils::rotor_from_angles(dy, dx, 0.0);

        camera.transformation.append_rotation(offset_rotor);
    }

    /// I can feel the suffer on it's way back to us
    fn change_fov(&mut self, camera: &mut Camera) {

        let fov_angle = camera.fov;

        let mut fov: f32 = fov_angle - self.zoom / 5.0;

        // You want some flippy dippy or some zoomers?
        if fov >= 180.0 || fov <  0.0 {
            // how about no
            if fov >= 180.0 {
                fov -= fov_angle
            } else {
                fov += fov_angle
            };
            return;
        }


        println!("fov_angle: {}", fov);

        // Don't even complain about the Variable names, I didn't
        // come up with them >:c
        let t: f32 = (fov / 2.0).tan();
        let sy: f32 = 1.0 / t;
        let sx: f32 = sy / camera.aspect;

        camera.perspective.cols[0].x = sx;
        camera.perspective.cols[1].y = sy;
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
            zoom: 0.0,
            old_mouse_coords: (0.0, 0.0),
            mouse_coords: (0.0, 0.0),
        }
    }
}