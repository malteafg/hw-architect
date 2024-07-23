use super::camera_controller::CameraController;

use gfx_api::GfxSuper;
use tool::ToolHandler;
use utils::input::{self, InputState};

use glam::*;

use std::time::Duration;

pub struct State<G: GfxSuper> {
    gfx_handle: G,
    window_width: u32,
    window_height: u32,
    camera_controller: CameraController,
    tool: ToolHandler<G, world::World>,
    input_state: InputState,
    ground_pos: Vec3,
}

impl<G: GfxSuper> State<G> {
    pub fn new(mut gfx_handle: G, window_width: u32, window_height: u32) -> Self {
        let camera_controller = CameraController::new(
            Vec3::new(0.0, 0.0, 0.0),
            50.0f32.to_radians(),
            2.0f32.to_radians(),
            100.0,
        );

        let world = world::World::new();
        let tool = ToolHandler::new(&mut gfx_handle, world);

        Self {
            gfx_handle,
            window_width,
            window_height,
            camera_controller,
            tool,
            input_state: InputState::default(),
            ground_pos: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn key_input(&mut self, action: input::KeyAction) {
        self.camera_controller.process_keyboard(action);
        self.tool.process_keyboard(&mut self.gfx_handle, action);
    }

    pub fn mouse_input(&mut self, event: input::MouseEvent) {
        self.camera_controller.process_mouse(event);
        match event {
            input::MouseEvent::Dragged(_, mouse_pos, _)
            | input::MouseEvent::Moved(mouse_pos, _) => {
                self.input_state.mouse_pos = mouse_pos;
                self.update_ground_pos();
            }
            _ => {}
        };

        self.tool.mouse_input(&mut self.gfx_handle, event);
    }

    pub fn update(&mut self, dt: Duration) {
        if self.camera_controller.update_camera(dt) {
            self.update_ground_pos();
        }
        self.tool.update(dt);
        self.gfx_handle
            .update_camera(self.camera_controller.get_raw_camera());
        self.gfx_handle.update(dt);
    }

    fn update_ground_pos(&mut self) {
        let ray_dir = self.gfx_handle.compute_ray(
            [
                self.input_state.mouse_pos.x as f32,
                self.input_state.mouse_pos.y as f32,
            ],
            self.camera_controller.get_raw_camera(),
        );
        let ray_dir = Vec3::from_array(ray_dir);
        let cam_pos = self.camera_controller.get_camera_pos();
        let ground_pos = cam_pos + ray_dir * (-cam_pos.y / ray_dir.y);
        self.ground_pos = ground_pos;
        self.tool
            .update_ground_pos(&mut self.gfx_handle, self.ground_pos);
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        self.gfx_handle.resize(window_width, window_height);
    }

    pub fn redraw(&mut self) {
        self.resize(self.window_width, self.window_height);
    }

    pub fn render(&mut self) -> Result<(), gfx_api::GfxError> {
        self.gfx_handle.render()
    }
}
