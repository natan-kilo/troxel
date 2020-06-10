use na::{
    Quaternion,
    Unit,
    geometry::UnitQuaternion,
    Vector2,
    Vector3,
    Vector4,
    Perspective3,
    Projective3,
    Translation3,
    Matrix4,
};

use winit_input_helper::WinitInputHelper;
use winit::event::VirtualKeyCode as vkc;
use cgmath::num_traits::real::Real;

use std::f32::consts::FRAC_PI_4;

pub struct Camera {
    perspective: Perspective3<f32>,
    position: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
}

impl Camera {
    pub fn new(sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let aspect = sc_desc.width as f32 / sc_desc.height as f32;
        let fov_y: f32 = std::f32::consts::PI / 3.0;
        let z_near: f32 = 0.1;
        let z_far: f32 = 100.0;
        let perspective = Perspective3::new(aspect, fov_y, z_near, z_far);
        let position = Vector3::new(0.0, -0.0, -2.0);
        let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

        Self {
            perspective,
            position,
            rotation,
        }
    }

    pub fn update_position(&mut self, position: &Vector3<f32>) {
        self.position = position.clone();
    }

    pub fn update_rotation(&mut self, rotation: &UnitQuaternion<f32>) {
        self.rotation = rotation.clone();
    }


    fn projection_matrix(&self) -> Matrix4<f32> {
        let proj = self.perspective.clone();
        proj.into_inner()
    }

    fn view_matrix(&self) -> Matrix4<f32> {
        let rotation: Matrix4<f32> = self.rotation.into();
        let translation_matrix: Matrix4<f32> = Matrix4::new_translation(&self.position);
        rotation * translation_matrix
    }

    pub fn to_matrix(&self) -> Matrix4<f32> {
        let projection = self.projection_matrix();
        let view = self.view_matrix();
        projection * view
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
    old_mouse_coords: Vector2<f32>,
    mouse_coords: Vector2<f32>,
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
                Vector2::new(coords.0, coords.1)
            }
            _ => {
                Vector2::new(0.0, 0.0)
            }
        }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        let mut new_position: Vector3<f32> = camera.position.clone();
        let current_rotation: UnitQuaternion<f32> = camera.rotation.clone().into();
        let angles: (f32, f32, f32) = current_rotation.euler_angles();
        let angle_vec: Vector3<f32> = Vector3::new(angles.1, angles.0, angles.2).normalize();
        let mut speed = self.speed;

        let pi_2 = std::f32::consts::FRAC_PI_2;

        match self.is_con {
            true => {
                speed *= 2.0;
            }
            false => { }
        }

        match self.is_w {
            true => {
                new_position.data[2] -= speed * angle_vec.data[0];
                new_position.data[0] -= speed * angle_vec.data[0];
            }
            false => {}
        }
        match self.is_s {
            true => {
                new_position.data[2] += angles.1 * speed;
                new_position.data[0] += angles.1 * speed;
            }
            false => {}
        }
        match self.is_a {
            true => {
                // new_position.data[2] -= (angles.0 + pi_2).cos() * speed;
                // new_position.data[0] -= -(angles.data[0] + pi_2).sin() * speed;
            }
            false => {}
        }
        match self.is_d {
            true => {
                // new_position.data[2] += (angles.data[0] + pi_2).cos() * speed;
                // new_position.data[0] += -(angles.data[0] + pi_2).sin() * speed;
            }
            false => {}
        }
        match self.is_space {
            true => {
                new_position.data[1] += -speed;
            }
            false => {}
        }
        match self.is_shift {
            true => {
                new_position.data[1] += speed;
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

        let mouse_delta: Vector2<f32> = self.mouse_coords.clone() - self.old_mouse_coords.clone();
        self.old_mouse_coords = self.mouse_coords.clone();

        camera.update_position(&new_position);
        let new_rotation = self.update_rotation(&mouse_delta, current_rotation);
        camera.update_rotation(&new_rotation);
    }

    pub fn update_rotation(&mut self, target: &Vector2<f32>, current_rotation: UnitQuaternion<f32>) -> UnitQuaternion<f32> {
        if target.magnitude_squared() == 0.0 {
            current_rotation;
        }

        let roll: f32 = target.data[1] / 100.0;
        let pitch: f32 = target.data[0] / 100.0;

        let current_euler: (f32, f32, f32) = current_rotation.euler_angles().clone();

        let target_euler: (f32, f32, f32) = (target.data[0] / 100.0, target.data[1] / 100.0, 0.0);
        let new_euler: (f32, f32, f32) = (current_euler.0 + target_euler.1, current_euler.1 + target_euler.0, current_euler.2);

        println!("{:?}; {:?}", current_euler, new_euler);
        let new_rotation = UnitQuaternion::from_euler_angles(new_euler.0,  new_euler.1, new_euler.2 );
        // let new_rotation: UnitQuaternion<f32> = UnitQuaternion::from_euler_angles(new_euler.0, new_euler.1, new_euler.2);

        new_rotation
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
            old_mouse_coords: Vector2::new(0.0, 0.0),
            mouse_coords: Vector2::new(0.0, 0.0),
        }
    }
}