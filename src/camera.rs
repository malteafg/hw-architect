use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;

// struct Camera {
//     eye: cgmath::Point3<f32>,
//     target: cgmath::Point3<f32>,
//     up: cgmath::Vector3<f32>,
//     aspect: f32,
//     fovy: f32,
//     znear: f32,
//     zfar: f32,
// }

// impl Camera {
//     fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
//         let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
//         let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
//         return OPENGL_TO_WGPU_MATRIX * proj * view;
//     }
// }

// struct CameraController {
//     speed: f32,
//     is_forward_pressed: bool,
//     is_backward_pressed: bool,
//     is_left_pressed: bool,
//     is_right_pressed: bool,
// }

// impl CameraController {
//     fn new(speed: f32) -> Self {
//         Self {
//             speed,
//             is_forward_pressed: false,
//             is_backward_pressed: false,
//             is_left_pressed: false,
//             is_right_pressed: false,
//         }
//     }

//     fn process_events(&mut self, event: &WindowEvent) -> bool {
//         match event {
//             WindowEvent::KeyboardInput {
//                 input:
//                     KeyboardInput {
//                         state,
//                         virtual_keycode: Some(keycode),
//                         ..
//                     },
//                 ..
//             } => {
//                 let is_pressed = *state == ElementState::Pressed;
//                 match keycode {
//                     VirtualKeyCode::W | VirtualKeyCode::Up => {
//                         self.is_forward_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::A | VirtualKeyCode::Left => {
//                         self.is_left_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::S | VirtualKeyCode::Down => {
//                         self.is_backward_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::D | VirtualKeyCode::Right => {
//                         self.is_right_pressed = is_pressed;
//                         true
//                     }
//                     _ => false,
//                 }
//             }
//             _ => false,
//         }
//     }

//     fn update_camera(&self, camera: &mut Camera) {
//         let forward = camera.target - camera.eye;
//         let forward_norm = forward.normalize();
//         let forward_mag = forward.magnitude();

//         // Prevents glitching when camera gets too close to the
//         // center of the scene.
//         if self.is_forward_pressed && forward_mag > self.speed {
//             camera.eye += forward_norm * self.speed;
//         }
//         if self.is_backward_pressed {
//             camera.eye -= forward_norm * self.speed;
//         }

//         let right = forward_norm.cross(camera.up);

//         // Redo radius calc in case the fowrard/backward is pressed.
//         let forward = camera.target - camera.eye;
//         let forward_mag = forward.magnitude();

//         if self.is_right_pressed {
//             // Rescale the distance between the target and eye so
//             // that it doesn't change. The eye therefore still
//             // lies on the circle made by the target and eye.
//             camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
//         }
//         if self.is_left_pressed {
//             camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
//         }
//     }
// }

// let camera = Camera {
//     // position the camera one unit up and 2 units back
//     // +z is out of the screen
//     eye: (0.0, 1.0, 2.0).into(),
//     // have it look at the origin
//     target: (0.0, 0.0, 0.0).into(),
//     // which way is "up"
//     up: cgmath::Vector3::unit_y(),
//     aspect: config.width as f32 / config.height as f32,
//     fovy: 45.0,
//     znear: 0.1,
//     zfar: 100.0,
// };

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}
