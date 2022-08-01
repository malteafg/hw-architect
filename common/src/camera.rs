use crate::input::{Action, KeyAction, MouseEvent};
use crate::math_utils::VecPoint;
use cgmath::*;
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

const MIN_CAMERA_PITCH: f32 = 0.15;
const MAX_CAMERA_PITCH: f32 = 1.5;
const MIN_CAMERA_DIST: f32 = 30.0;
const MAX_CAMERA_DIST: f32 = 700.0;
const MAX_CAMERA_SPEED: f32 = 10.0;
const CAMERA_MOVE_SPEED: f32 = 0.05;
const CAMERA_MOVE_SMOOTH_FACTOR: f32 = 10.0;
const MOUSE_SENSITIVITY: f32 = 0.001;

const NUM_OF_MOVE_BUTTONS: i32 = 6;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

// const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub target: Vector3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    dist_to_target: f32,
}

impl Camera {
    pub fn new<V: Into<Vector3<f32>>, R: Into<Rad<f32>>>(
        target: V,
        yaw: R,
        pitch: R,
        dist_to_target: f32,
    ) -> Self {
        Self {
            target: target.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            dist_to_target,
        }
    }

    pub fn calc_pos(&self) -> Point3<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        (self.target
            + (Vector3::new(-cos_yaw, 0.0, -sin_yaw) * cos_pitch
                + Vector3::new(0.0, sin_pitch, 0.0))
                * self.dist_to_target)
            .to_point3()
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.calc_pos(),
            Vector3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw).normalize(),
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
    input: [bool; NUM_OF_MOVE_BUTTONS as usize],
    velocity: [i32; (2 + NUM_OF_MOVE_BUTTONS) as usize],
    delta_pitch: Rad<f32>,
    delta_yaw: Rad<f32>,
    next_pitch: Rad<f32>,
    next_yaw: Rad<f32>,
    next_dist: f32,
    next_target: Vector3<f32>,
    progress: f32,
    progression_speed: f32,
    progression_function: fn(f32) -> f32,
    move_init: bool,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            input: [false; NUM_OF_MOVE_BUTTONS as usize],
            velocity: [0; (2 + NUM_OF_MOVE_BUTTONS) as usize],
            delta_pitch: Rad(0.0),
            delta_yaw: Rad(0.0),
            next_pitch: Rad(0.0),
            next_yaw: Rad(0.0),
            next_dist: 0.0,
            next_target: Vector3::new(0.0, 0.0, 0.0),
            progress: 0.0,
            progression_speed: 0.0,
            progression_function: CameraController::smooth_move,
            move_init: false,
        }
    }

    pub fn linear_move(f: f32) -> f32 {
        f
    }

    pub fn smooth_move(f: f32) -> f32 {
        (CAMERA_MOVE_SMOOTH_FACTOR + 1.0) / 2.0 * (2.0 * f - 1.0)
            / f32::sqrt(
                CAMERA_MOVE_SMOOTH_FACTOR.powi(2) * ((2.0 * f - 1.0).powi(2) - 1.0)
                    + (CAMERA_MOVE_SMOOTH_FACTOR + 1.0).powi(2),
            )
            + 0.5
    }

    pub fn polynomial_move(f: f32) -> f32 {
        3.0 * f.powi(2) - 2.0 * f.powi(3)
    }

    pub fn momentum_move(f: f32) -> f32 {
        let a = 5.2; //6 is extremely cartoony, 5.2 is balanced
        a * f.powi(3)
            + 60.0 * f.powi(4)
            + (-3.0 * a - 184.0) * f.powi(5)
            + (2.0 * a + 195.0) * f.powi(6)
            - 70.0 * f.powi(7)
    }

    pub fn move_camera(
        &mut self,
        target: Vector3<f32>,
        p: Rad<f32>,
        y: Rad<f32>,
        d: f32,
        speed: f32,
        func: fn(f32) -> f32,
    ) {
        self.next_target = target;
        self.next_pitch = Rad(restrainf(p.0, MIN_CAMERA_PITCH, MAX_CAMERA_PITCH));
        self.next_yaw = y.normalize();
        self.next_dist = restrainf(d, MIN_CAMERA_DIST, MAX_CAMERA_DIST);
        self.progression_speed = speed;
        self.progression_function = func;
        self.move_init = true;
    }

    fn stop_move_progression(&mut self) {
        self.progression_speed = 0.0;
        self.progress = 0.0;
    }

    pub fn process_keyboard(&mut self, key: KeyAction) -> bool {
        match key {
            (Action::CameraUp, pressed) => {
                self.input[0] = pressed;
                true
            }
            (Action::CameraDown, pressed) => {
                self.input[1] = pressed;
                true
            }
            (Action::CameraLeft, pressed) => {
                self.input[2] = pressed;
                true
            }
            (Action::CameraRight, pressed) => {
                self.input[3] = pressed;
                true
            }
            (Action::CameraRotateLeft, pressed) => {
                self.input[4] = pressed;
                true
            }
            (Action::CameraRotateRight, pressed) => {
                self.input[5] = pressed;
                true
            }
            _ => false,
            // VirtualKeyCode::Space => {
            //     self.move_camera(
            //         Vector3::new(0.0, 0.0, 0.0),
            //         Rad(PI / 4.0),
            //         Rad(1.0),
            //         100.0,
            //         1.0,
            //         CameraController::polynomial_move,
            //     );
            //     false
            // }
            // _ => false,
        }

        // if pressed && key_matched {
        //     self.stop_move_progression();
        // }

        // key_matched
    }

    pub fn process_mouse(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::Dragged { delta, .. } => {
                self.delta_yaw += Rad(-MOUSE_SENSITIVITY * delta.dx as f32);
                self.delta_pitch += Rad(MOUSE_SENSITIVITY * delta.dy as f32);
                self.stop_move_progression();
            }
            MouseEvent::Scrolled(scroll) => {
                self.velocity[7] += (2.0 * scroll) as i32;
                self.stop_move_progression();
            }
            _ => {}
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        if self.progression_speed > 0.0 {
            self.update_progress(camera, dt)
        } else {
            self.update_manuel(camera, dt)
        }
    }

    fn update_progress(&mut self, camera: &mut Camera, dt: f32) {
        if self.move_init {
            self.move_init = false;
            self.progression_speed *= 36.0
                / (300.0
                    + camera.target.distance(self.next_target)
                    + (camera.dist_to_target - self.next_dist).abs())
                .sqrt();

            let yaw_diff = (self.next_yaw.0 - camera.yaw.0).abs();
            if yaw_diff.abs() >= PI {
                camera.yaw = camera.yaw.normalize();
                let dir = if camera.yaw > self.next_yaw {
                    -1.0
                } else {
                    1.0
                };
                camera.yaw += if (self.next_yaw.0 - camera.yaw.0).abs() > PI {
                    Rad(dir * PI)
                } else {
                    Rad(0.0)
                };
            }
        }

        let old_progress = (self.progression_function)(self.progress);
        self.progress = f32::min(self.progress + self.progression_speed * dt, 1.0);
        let new_progress = (self.progression_function)(self.progress);

        camera.pitch = Rad(interpolate(
            old_progress,
            new_progress,
            camera.pitch.0,
            self.next_pitch.0,
        ));
        camera.yaw = Rad(interpolate(
            old_progress,
            new_progress,
            camera.yaw.0,
            self.next_yaw.0,
        ));
        camera.dist_to_target = interpolate(
            old_progress,
            new_progress,
            camera.dist_to_target,
            self.next_dist,
        );
        camera.target =
            interpolate_vector(old_progress, new_progress, camera.target, self.next_target);

        if self.progress >= 1.0 {
            self.stop_move_progression()
        }
    }

    fn update_manuel(&mut self, camera: &mut Camera, dt: f32) {
        camera.yaw = Rad(center(camera.yaw.0 - self.delta_yaw.0, PI));
        camera.pitch = Rad(restrainf(
            camera.pitch.0 + self.delta_pitch.0,
            MIN_CAMERA_PITCH,
            MAX_CAMERA_PITCH,
        ));

        self.delta_yaw = Rad(0.0);
        self.delta_pitch = Rad(0.0);

        for i in 0..NUM_OF_MOVE_BUTTONS as usize {
            self.velocity[i] = restrain(
                self.velocity[i] + bool_to_int(self.input[i]),
                0,
                MAX_CAMERA_SPEED as i32,
            );
        }

        let speed = CAMERA_MOVE_SPEED * camera.dist_to_target * dt;
        if self.velocity[0] > 0 {
            camera.target = camera.target
                + (calc_direction_vector(camera.yaw.0 + PI) * speed * (self.velocity[0] as f32))
        }
        if self.velocity[1] > 0 {
            camera.target = camera.target
                + (calc_direction_vector(camera.yaw.0) * speed * (self.velocity[1] as f32))
        }
        if self.velocity[2] > 0 {
            camera.target = camera.target
                + (calc_direction_vector(camera.yaw.0 + FRAC_PI_2)
                    * speed
                    * (self.velocity[2] as f32))
        }
        if self.velocity[3] > 0 {
            camera.target = camera.target
                + (calc_direction_vector(camera.yaw.0 - FRAC_PI_2)
                    * speed
                    * (self.velocity[3] as f32))
        }

        if camera.target.magnitude() > 500.0 {
            camera.target = camera.target.normalize() * 500.0;
        }

        if self.velocity[4] > 0 {
            camera.yaw = Rad(center(
                camera.yaw.0 + (self.velocity[4] as f32) * 5.0 * CAMERA_MOVE_SPEED * dt,
                PI,
            ));
        }
        if self.velocity[5] > 0 {
            camera.yaw = Rad(center(
                camera.yaw.0 - (self.velocity[5] as f32) * 5.0 * CAMERA_MOVE_SPEED * dt,
                PI,
            ));
        }

        if self.velocity[6] != 0 || self.velocity[7] != 0 {
            self.velocity[6] +=
                if self.velocity[7] > 0 || (self.velocity[7] == 0 && self.velocity[6] < 0) {
                    1
                } else {
                    -1
                };
            let dist = new_dist(camera.dist_to_target, conseq_sum(self.velocity[6]));

            if self.velocity[6].abs() as f32 >= (self.velocity[7].abs() as f32).sqrt()
                || dist < MIN_CAMERA_DIST
                || dist > MAX_CAMERA_DIST
            {
                self.velocity[7] = 0;
            }
            camera.dist_to_target = restrainf(
                new_dist(camera.dist_to_target, self.velocity[6]),
                MIN_CAMERA_DIST,
                MAX_CAMERA_DIST,
            );
        }
    }
}

/// Interpolates between an intermidary vector and the target vector
fn interpolate(old_progress: f32, new_progress: f32, old_value: f32, target_value: f32) -> f32 {
    if old_progress == 1.0 {
        old_value
    } else {
        target_value * new_progress
            + (old_value - target_value * old_progress)
                * ((1.0 - new_progress) / (1.0 - old_progress))
    }
}

fn interpolate_vector(
    old_progress: f32,
    new_progress: f32,
    old_vector: Vector3<f32>,
    target_vector: Vector3<f32>,
) -> Vector3<f32> {
    if old_progress == 1.0 {
        old_vector
    } else {
        target_vector * new_progress
            + (old_vector - target_vector * old_progress)
                * ((1.0 - new_progress) / (1.0 - old_progress))
    }
}

fn restrain(value: i32, min: i32, max: i32) -> i32 {
    min.max(value.min(max))
}

fn restrainf(value: f32, min: f32, max: f32) -> f32 {
    min.max(value.min(max))
}

fn bool_to_int(b: bool) -> i32 {
    if b {
        1
    } else {
        -1
    }
}

fn calc_direction_vector(angle: f32) -> Vector3<f32> {
    let (sin_yaw, cos_yaw) = angle.sin_cos();
    Vector3::new(-cos_yaw, 0.0, -sin_yaw)
}

fn center(v: f32, r: f32) -> f32 {
    let f = v % (2.0 * r);
    if f > r {
        f - 2.0 * r
    } else if f < -r {
        f + 2.0 * r
    } else {
        f
    }
}

fn new_dist(dist: f32, velocity: i32) -> f32 {
    dist * f32::powf(1.03, velocity as f32)
}

fn conseq_sum(value: i32) -> i32 {
    value * (value.abs() + 1) / 2
}
