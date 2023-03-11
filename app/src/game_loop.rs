use super::{configuration, input_handler, state};

use utils::input;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use glam::*;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
    let window_width = config.window.width as u32;
    let window_height = config.window.height as u32;
    let key_map = configuration::load_key_map(config.key_map).await.unwrap();
    let input_handler = input_handler::InputHandler::new(key_map);

    // create event_loop and window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Highway Architect");
    window.set_inner_size(PhysicalSize::new(window_width, window_height));
    window.set_outer_position(PhysicalPosition::new(0, 0));

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_inner_size(PhysicalSize::new(window_width, window_height));

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

    // Create handle to graphics card. Change line to use different gpu backend.
    let gfx = gfx_wgpu::GfxState::new(&window, window_width, window_height).await;
    let mut state = state::State::new(gfx, window_width, window_height, input_handler).await;

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
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        #[cfg(not(target_arch = "wasm32"))]
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size.width, physical_size.height);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(new_inner_size.width, new_inner_size.height);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    state.update(dt);
                    let render_error = state.render();

                    use gfx_api::GfxFrameError;
                    match render_error {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(GfxFrameError::Lost | GfxFrameError::Outdated) => {
                            state.redraw();
                        }
                        // The system is out of memory, we should probably quit
                        Err(GfxFrameError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // We're ignoring timeouts
                        Err(GfxFrameError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                _ => {}
            },
        }
    });
}
