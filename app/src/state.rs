use super::camera_controller::CameraController;
use super::input_handler::InputHandler;

use gfx_api::GfxSuper;
use tool::ToolHandler;
use utils::input;

use glam::*;
use winit::window::Window;

use std::time::Duration;

pub struct State<'window, G: GfxSuper> {
    gfx_handle: G,
    window: &'window Window,
    window_width: u32,
    window_height: u32,
    camera_controller: CameraController,
    pub input_handler: InputHandler,
    tool: ToolHandler<G>,
    ground_pos: Vec3,
}

impl<'window, G: GfxSuper> State<'window, G> {
    pub fn new(
        mut gfx_handle: G,
        window: &'window Window,
        window_width: u32,
        window_height: u32,
        input_handler: InputHandler,
    ) -> Self {
        let camera_controller = CameraController::new(
            Vec3::new(0.0, 0.0, 0.0),
            50.0f32.to_radians(),
            2.0f32.to_radians(),
            100.0,
        );

        let world = world::World::new();
        let tool = ToolHandler::new(&mut gfx_handle, Box::new(world));

        Self {
            gfx_handle,
            window,
            window_width,
            window_height,
            camera_controller,
            input_handler,
            tool,
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
            input::MouseEvent::Dragged(_, _) | input::MouseEvent::Moved(_) => {
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
        let mouse_pos = self.input_handler.get_mouse_pos();
        let ray_dir = self.gfx_handle.compute_ray(
            [mouse_pos.x as f32, mouse_pos.y as f32],
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

    pub fn render(&mut self) -> Result<(), gfx_api::GfxFrameError> {
        self.gfx_handle.render()
    }

    pub fn window(&self) -> &'window Window {
        &self.window
    }
}
