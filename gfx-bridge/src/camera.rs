use glam::*;
use utils::{Angle, Mat4Utils, input};
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

const MIN_CAMERA_PITCH: f32 = 0.15;
const MAX_CAMERA_PITCH: f32 = 1.5;
const MIN_CAMERA_DIST: f32 = 30.0;
const MAX_CAMERA_DIST: f32 = 700.0;
const MAX_CAMERA_SPEED: f32 = 10.0;
const CAMERA_MOVE_SPEED: f32 = 0.05;
const CAMERA_MOVE_SMOOTH_FACTOR: f32 = 10.0;

const MOUSE_HORIZONTAL_SENSITIVITY: f32 = 0.003;
const MOUSE_VERTICAL_SENSITIVITY: f32 = 0.002;

const NUM_OF_MOVE_BUTTONS: i32 = 6;

// const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub target: Vec3,
    yaw: f32,
    pitch: f32,
    dist_to_target: f32,
}

impl Camera {
    pub fn new(target: Vec3, yaw: f32, pitch: f32, dist_to_target: f32) -> Self {
        Self {
            target,
            yaw,
            pitch,
            dist_to_target,
        }
    }

    pub fn calc_pos(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        self.target
            + (Vec3::new(-cos_yaw, 0.0, -sin_yaw) * cos_pitch + Vec3::new(0.0, sin_pitch, 0.0))
                * self.dist_to_target
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_to_rh(
            self.calc_pos(),
            Vec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    input: [bool; NUM_OF_MOVE_BUTTONS as usize],
    velocity: [i32; (2 + NUM_OF_MOVE_BUTTONS) as usize],
    delta_pitch: f32,
    delta_yaw: f32,
    next_pitch: f32,
    next_yaw: f32,
    next_dist: f32,
    next_target: Vec3,
    progress: f32,
    progression_speed: f32,
    progression_function: fn(f32) -> f32,
    move_init: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            input: [false; NUM_OF_MOVE_BUTTONS as usize],
            velocity: [0; (2 + NUM_OF_MOVE_BUTTONS) as usize],
            delta_pitch: 0.0,
            delta_yaw: 0.0,
            next_pitch: 0.0,
            next_yaw: 0.0,
            next_dist: 0.0,
            next_target: Vec3::new(0.0, 0.0, 0.0),
            progress: 0.0,
            progression_speed: 0.0,
            progression_function: CameraController::smooth_move,
            move_init: false,
        }
    }
}

impl CameraController {
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
        target: Vec3,
        p: f32,
        y: f32,
        d: f32,
        speed: f32,
        func: fn(f32) -> f32,
    ) {
        self.next_target = target;
        self.next_pitch = restrainf(p, MIN_CAMERA_PITCH, MAX_CAMERA_PITCH);
        self.next_yaw = y.rad_normalize();
        self.next_dist = restrainf(d, MIN_CAMERA_DIST, MAX_CAMERA_DIST);
        self.progression_speed = speed;
        self.progression_function = func;
        self.move_init = true;
    }

    fn stop_move_progression(&mut self) {
        self.progression_speed = 0.0;
        self.progress = 0.0;
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) -> bool {
        use input::Action::*;
        match key {
            (CameraUp, pressed) => {
                self.input[0] = pressed;
                true
            }
            (CameraDown, pressed) => {
                self.input[1] = pressed;
                true
            }
            (CameraLeft, pressed) => {
                self.input[2] = pressed;
                true
            }
            (CameraRight, pressed) => {
                self.input[3] = pressed;
                true
            }
            (CameraRotateLeft, pressed) => {
                self.input[4] = pressed;
                true
            }
            (CameraRotateRight, pressed) => {
                self.input[5] = pressed;
                true
            }
            (CameraReturn, pressed) if pressed => {
                self.move_camera(
                    Vec3::new(0.0, 0.0, 0.0),
                    PI / 4.0,
                    1.0,
                    100.0,
                    1.0,
                    CameraController::polynomial_move,
                );
                true
            }
            _ => false,
        }

        // if pressed && key_matched {
        //     self.stop_move_progression();
        // }

        // key_matched
    }

    pub fn process_mouse(&mut self, event: input::MouseEvent) {
        match event {
            input::MouseEvent::Dragged(button, delta) if button == input::Mouse::Middle => {
                self.delta_yaw += -MOUSE_HORIZONTAL_SENSITIVITY * delta.dx as f32;
                self.delta_pitch += MOUSE_VERTICAL_SENSITIVITY * delta.dy as f32;
                self.stop_move_progression();
            }
            input::MouseEvent::Scrolled(scroll) => {
                self.velocity[7] += (2.0 * scroll) as i32;
                self.stop_move_progression();
            }
            _ => {}
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) -> bool {
        let dt = dt.as_secs_f32();

        let pos = camera.calc_pos();
        if self.progression_speed > 0.0 {
            self.update_progress(camera, dt)
        } else {
            self.update_manuel(camera, dt)
        }
        pos != camera.calc_pos()
    }

    fn update_progress(&mut self, camera: &mut Camera, dt: f32) {
        if self.move_init {
            self.move_init = false;
            self.progression_speed *= 36.0
                / (300.0
                    + camera.target.distance(self.next_target)
                    + (camera.dist_to_target - self.next_dist).abs())
                .sqrt();

            let yaw_diff = (self.next_yaw - camera.yaw).abs();
            if yaw_diff.abs() >= PI {
                camera.yaw = camera.yaw.rad_normalize();
                let dir = if camera.yaw > self.next_yaw {
                    -1.0
                } else {
                    1.0
                };
                camera.yaw += if (self.next_yaw - camera.yaw).abs() > PI {
                    dir * PI
                } else {
                    0.0
                };
            }
        }

        let old_progress = (self.progression_function)(self.progress);
        self.progress = f32::min(self.progress + self.progression_speed * dt, 1.0);
        let new_progress = (self.progression_function)(self.progress);

        camera.pitch = interpolate(old_progress, new_progress, camera.pitch, self.next_pitch);
        camera.yaw = interpolate(old_progress, new_progress, camera.yaw, self.next_yaw);
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
        camera.yaw = center(camera.yaw - self.delta_yaw, PI);
        camera.pitch = restrainf(
            camera.pitch + self.delta_pitch,
            MIN_CAMERA_PITCH,
            MAX_CAMERA_PITCH,
        );

        self.delta_yaw = 0.0;
        self.delta_pitch = 0.0;

        for i in 0..NUM_OF_MOVE_BUTTONS as usize {
            self.velocity[i] = restrain(
                self.velocity[i] + bool_to_int(self.input[i]),
                0,
                MAX_CAMERA_SPEED as i32,
            );
        }

        let speed = CAMERA_MOVE_SPEED * camera.dist_to_target * dt;
        if self.velocity[0] > 0 {
            camera.target +=
                calc_direction_vector(camera.yaw + PI) * speed * (self.velocity[0] as f32)
        }
        if self.velocity[1] > 0 {
            camera.target += calc_direction_vector(camera.yaw) * speed * (self.velocity[1] as f32)
        }
        if self.velocity[2] > 0 {
            camera.target +=
                calc_direction_vector(camera.yaw + FRAC_PI_2) * speed * (self.velocity[2] as f32)
        }
        if self.velocity[3] > 0 {
            camera.target +=
                calc_direction_vector(camera.yaw - FRAC_PI_2) * speed * (self.velocity[3] as f32)
        }

        if camera.target.length() > 500.0 {
            camera.target = camera.target.normalize() * 500.0;
        }

        if self.velocity[4] > 0 {
            camera.yaw = center(
                camera.yaw + (self.velocity[4] as f32) * 5.0 * CAMERA_MOVE_SPEED * dt,
                PI,
            );
        }
        if self.velocity[5] > 0 {
            camera.yaw = center(
                camera.yaw - (self.velocity[5] as f32) * 5.0 * CAMERA_MOVE_SPEED * dt,
                PI,
            );
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
    old_vector: Vec3,
    target_vector: Vec3,
) -> Vec3 {
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

fn calc_direction_vector(angle: f32) -> Vec3 {
    let (sin_yaw, cos_yaw) = angle.sin_cos();
    Vec3::new(-cos_yaw, 0.0, -sin_yaw)
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
