mod camera_controller;
mod configuration;
mod input_handler;

use camera_controller::CameraController;
use gfx_api::Gfx;
use tool::WorldTool;
use utils::input;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use glam::*;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use std::cell::RefCell;
use std::rc::Rc;

struct State {
    /// The handle to the graphics card. A reference counter is used such that the road tool can
    /// also have a reference to the gfx_handle. Functionality is still separated as road tool only
    /// uses the GfxRoadData trait.
    gfx_handle: Rc<RefCell<gfx_wgpu::GfxState>>,
    window_size: PhysicalSize<u32>,
    camera: gfx_api::Camera,
    camera_controller: CameraController,
    input_handler: input_handler::InputHandler,
    tool: WorldTool,
    ground_pos: Vec3,
}

impl State {
    async fn new(window: &Window, input_handler: input_handler::InputHandler) -> Self {
        // change line to use other gpu backend
        let gfx = gfx_wgpu::GfxState::new(window).await;
        let window_size = window.inner_size();

        let camera = gfx_api::Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            2.0f32.to_radians(),
            50.0f32.to_radians(),
            100.0,
        );
        let camera_controller = CameraController::default();

        let gfx_handle = Rc::new(RefCell::new(gfx));
        let gfx_handle_tool = Rc::clone(&gfx_handle);

        Self {
            gfx_handle,
            window_size,
            camera,
            camera_controller,
            input_handler,
            tool: WorldTool::new(gfx_handle_tool, simulation::World::new()),
            ground_pos: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    fn key_input(&mut self, action: input::KeyAction) {
        self.camera_controller.process_keyboard(action);
        self.tool.process_keyboard(action);
    }

    fn mouse_input(&mut self, event: input::MouseEvent) {
        self.camera_controller.process_mouse(event);
        match event {
            input::MouseEvent::Dragged(_, _) | input::MouseEvent::Moved(_) => {
                self.update_ground_pos();
            }
            _ => {}
        };

        self.tool.mouse_input(event);
    }

    fn update(&mut self, dt: instant::Duration) {
        if self.camera_controller.update_camera(&mut self.camera, dt) {
            self.update_ground_pos();
        }
        use gfx_api::GfxCameraData;
        self.gfx_handle.borrow_mut().update_camera(&self.camera);
        self.gfx_handle.borrow_mut().update(dt);
    }

    fn update_ground_pos(&mut self) {
        let mouse_pos = self.input_handler.get_mouse_pos();
        use gfx_api::GfxCameraData;
        let ray = self.gfx_handle.borrow_mut().compute_ray(
            glam::Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32),
            &self.camera,
        );
        let ground_pos = ray.pos + ray.dir * (-ray.pos.y / ray.dir.y);
        self.ground_pos = ground_pos;
        self.tool.update_ground_pos(self.ground_pos);
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.gfx_handle.borrow_mut().resize(new_size);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // initialize logging depending on architecture
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // load configuration
    let config = configuration::load_config().await.unwrap();
    let key_map = configuration::load_key_map(config.key_map).await.unwrap();
    let input_handler = input_handler::InputHandler::new(key_map);

    // create event_loop and window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Highway Architect");
    window.set_inner_size(PhysicalSize::new(config.window.width, config.window.height));
    window.set_outer_position(PhysicalPosition::new(0, 0));

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_inner_size(PhysicalSize::new(config.window.width, config.window.height));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-container")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window, input_handler).await;

    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        use input::Action;
        use input_handler::InputEvent;
        match state.input_handler.process_input(&event, window.id()) {
            InputEvent::KeyAction(a) => match a {
                (Action::Exit, _) => *control_flow = ControlFlow::Exit,
                _ => state.key_input(a),
            },
            InputEvent::MouseEvent(e) => state.mouse_input(e),
            InputEvent::Absorb => {}
            InputEvent::Proceed => match event {
                Event::MainEventsCleared => window.request_redraw(),
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    #[cfg(not(target_arch = "wasm32"))]
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    state.update(dt);
                    let render_error = state.gfx_handle.borrow_mut().render();
                    match render_error {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.window_size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // We're ignoring timeouts
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                _ => {}
            },
        }
    });
}
