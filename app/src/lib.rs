#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod configuration;
use cgmath::*;
use common::road::tool;
use common::{camera, input, math_utils::VecPoint};
use graphics::graphics::*;

struct State {
    gfx: GfxState,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    input_handler: input::InputHandler,
    road_tool: tool::ToolState,
    ground_pos: Vector3<f32>,
}

impl State {
    async fn new(window: &Window, input_handler: input::InputHandler) -> Self {
        let gfx = GfxState::new(window).await;

        let camera =
            camera::Camera::new((0.0, 0.0, 0.0), cgmath::Deg(2.0), cgmath::Deg(50.0), 100.0);
        let camera_controller = camera::CameraController::new();

        Self {
            gfx,
            camera,
            camera_controller,
            input_handler,
            road_tool: tool::ToolState::new(),
            ground_pos: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    fn key_input(&mut self, action: input::KeyAction) {
        self.camera_controller.process_keyboard(action);
        self.road_tool.process_keyboard(action);
    }

    fn mouse_input(&mut self, event: input::MouseEvent) {
        self.camera_controller.process_mouse(event);
        match event {
            input::MouseEvent::LeftDragged(_)
            | input::MouseEvent::Moved(_)
            | input::MouseEvent::MiddleDragged(_)
            | input::MouseEvent::RightDragged(_) => {
                self.update_ground_pos();
            }
            _ => {}
        };

        let (road_mesh, road_tool_mesh) = self.road_tool.mouse_input(event);
        match road_mesh {
            Some(mesh) => self.gfx.update_road_buffer(mesh),
            None => {}
        };
        match road_tool_mesh {
            Some(mesh) => self.gfx.update_road_tool_buffer(mesh),
            None => {}
        };

        // match event {
        //     input::MouseEvent::LeftClick => {
        //         self.gfx.add_instance(ground_pos.to_vec3());
        //     }
        //     input::MouseEvent::RightClick => self.gfx.remove_instance(),
        //     _ => {}
        // }
    }

    fn update(&mut self, dt: instant::Duration) {
        let camera_moved = self.camera_controller.update_camera(&mut self.camera, dt);
        if camera_moved {
            self.update_ground_pos();
        }
        self.gfx.update(dt, &self.camera);
    }

    fn update_ground_pos(&mut self) {
        let (ray, pos) = self
            .gfx
            .calc_ray(&self.camera, self.input_handler.get_mouse_pos());
        let ground_pos = pos + ray * (-pos.y / ray.y);
        self.ground_pos = ground_pos.to_vec3();
        let road_tool_mesh = self.road_tool.update_ground_pos(self.ground_pos);
        match road_tool_mesh {
            Some(mesh) => self.gfx.update_road_tool_buffer(mesh),
            None => {}
        };
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
    let input_handler = input::InputHandler::new(key_map);

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
        use input::{Action, InputEvent};
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
                        state.gfx.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.gfx.resize(**new_inner_size);
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    state.update(dt);
                    match state.gfx.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.gfx.resize(state.gfx.size)
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
