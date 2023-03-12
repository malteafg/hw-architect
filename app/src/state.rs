use super::camera_controller::CameraController;
use super::input_handler::InputHandler;

use gfx_api::GfxSuper;
use tool::WorldTool;
use utils::input;

use glam::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct State {
    /// The handle to the graphics card. A reference counter is used such that tools can
    /// also have a reference to the gfx_handle. Functionality is still separated as tools have
    /// specific traits for interacting with the gpu.
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,
    window_width: u32,
    window_height: u32,
    camera_controller: CameraController,
    pub input_handler: InputHandler,
    tool: WorldTool,
    ground_pos: Vec3,
}

impl State {
    pub fn new(
        gfx_handle: Rc<RefCell<dyn GfxSuper>>,
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

        let gfx_handle_tool = Rc::clone(&gfx_handle);

        let world = world::World::new();
        let tool = WorldTool::new(gfx_handle_tool, world);

        Self {
            gfx_handle,
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
        self.tool.process_keyboard(action);
    }

    pub fn mouse_input(&mut self, event: input::MouseEvent) {
        self.camera_controller.process_mouse(event);
        match event {
            input::MouseEvent::Dragged(_, _) | input::MouseEvent::Moved(_) => {
                self.update_ground_pos();
            }
            _ => {}
        };

        self.tool.mouse_input(event);
    }

    pub fn update(&mut self, dt: instant::Duration) {
        if self.camera_controller.update_camera(dt) {
            self.update_ground_pos();
        }
        self.gfx_handle
            .borrow_mut()
            .update_camera(self.camera_controller.get_raw_camera());
        self.gfx_handle.borrow_mut().update(dt);
    }

    fn update_ground_pos(&mut self) {
        let mouse_pos = self.input_handler.get_mouse_pos();
        let ray_dir = self.gfx_handle.borrow_mut().compute_ray(
            [mouse_pos.x as f32, mouse_pos.y as f32],
            self.camera_controller.get_raw_camera(),
        );
        let ray_dir = Vec3::from_array(ray_dir);
        let cam_pos = self.camera_controller.get_camera_pos();
        let ground_pos = cam_pos + ray_dir * (-cam_pos.y / ray_dir.y);
        self.ground_pos = ground_pos;
        self.tool.update_ground_pos(self.ground_pos);
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        self.gfx_handle
            .borrow_mut()
            .resize(window_width, window_height);
    }

    pub fn redraw(&mut self) {
        self.resize(self.window_width, self.window_height);
    }

    pub fn render(&mut self) -> Result<(), gfx_api::GfxFrameError> {
        self.gfx_handle.borrow_mut().render()
    }
}
