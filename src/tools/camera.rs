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


pub struct Camera {
    perspective: Perspective3<f32>,
    position: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
    angles: Vector3<f32>,
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
            angles: Vector3::new(0.0, 0.0, 0.0),
            aspect,
            fov_y,
            z_near,
            z_far,
        }
    }

    pub fn update_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn update_rotation(&mut self, target: &Vector2<f32>) {
        if target.magnitude_squared() == 0.0 {
            return;
        }
        let mut target: Vector3<f32> = Vector3::new(target.data[0], target.data[1], 0.0);
        target.data[1] = -target.data[1];
        let pi = std::f32::consts::PI;
        let pi_2 = std::f32::consts::FRAC_PI_2;
        let mut new_angles: Vector3<f32> = self.angles.clone() + target * 0.8 / 180.0 * pi;

        if new_angles.data[1] < -pi_2 {
            new_angles.data[1] = -pi_2
        }
        if new_angles.data[1] > pi_2 {
            new_angles.data[1] = pi_2
        }

        println!("angles: {:?}", new_angles);
        println!("target: {:?}", target);
        self.angles = new_angles;
        let direction_forward: Vector3<f32> = Vector3::new(
            self.angles.data[0].sin(),
            self.angles.data[1].sin(),
            self.angles.data[0].cos(),
        );

        let direction_right: Vector3<f32> = Vector3::new(
            (self.angles.data[0] - pi_2).sin(),
            self.angles.data[1].sin(),
            (self.angles.data[0] - pi_2).cos(),
        );

        let up: Vector3<f32> = direction_right.cross(&direction_forward);

        let rotation = UnitQuaternion::face_towards(&direction_forward, &up);

        self.rotation = rotation;
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
        let angles: Vector3<f32> = camera.angles.clone();

        let pi = std::f32::consts::PI;
        let pi_2 = std::f32::consts::FRAC_PI_2;

        match self.is_w {
            true => {
                new_position.data[2] += angles.data[0].cos() * self.speed;
                new_position.data[1] += -angles.data[1].sin() * self.speed;
                new_position.data[0] += -angles.data[0].sin() * self.speed;
            }
            false => {}
        }
        match self.is_s {
            true => {
                new_position.data[2] -= angles.data[0].cos() * self.speed;
                new_position.data[1] -= -angles.data[1].sin() * self.speed;
                new_position.data[0] -= -angles.data[0].sin() * self.speed;
            }
            false => {}
        }
        match self.is_a {
            true => {
                new_position.data[2] -= (angles.data[0] + pi_2).cos() * self.speed;
                new_position.data[1] -= -angles.data[1].sin() * self.speed;
                new_position.data[0] -= -(angles.data[0] + pi_2).sin() * self.speed;
            }
            false => {}
        }
        match self.is_d {
            true => {
                new_position.data[2] += (angles.data[0] + pi_2).cos() * self.speed;
                new_position.data[1] += -angles.data[1].sin() * self.speed;
                new_position.data[0] += -(angles.data[0] + pi_2).sin() * self.speed;
            }
            false => {}
        }
        match self.is_space {
            true => {
                new_position.data[2] += angles.data[0].cos() * self.speed;
                new_position.data[1] += -(angles.data[1] + pi_2).sin() * self.speed;
                new_position.data[0] += -angles.data[0].sin() * self.speed;
            }
            false => {}
        }
        match self.is_shift {
            true => {
                new_position.data[2] -= angles.data[0].cos() * self.speed;
                new_position.data[1] -= -(angles.data[1] + pi_2).sin() * self.speed;
                new_position.data[0] -= -angles.data[0].sin() * self.speed;
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

        self.old_mouse_coords = self.mouse_coords.into();

        camera.update_position(new_position);
        camera.update_rotation(&mouse_delta);
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
            is_space: false,
            is_shift: false,
            is_q: false,
            is_e: false,
            old_mouse_coords: Vector2::new(0.0, 0.0),
            mouse_coords: Vector2::new(0.0, 0.0),
        }
    }
}