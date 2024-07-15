use super::input_handler;

use utils::input;

use glam::*;
use winit::{dpi::PhysicalSize, event::*, event_loop::ControlFlow, window::WindowBuilder};

use std::time::Instant;

pub async fn run() {
    env_logger::init();

    // load configuration
    let config = hw_architect::config::load_config().unwrap();
    let window_width = config.window.width as u32;
    let window_height = config.window.height as u32;

    let key_map = input_handler::load_key_map(config.key_map).unwrap();
    let mut input_handler = input_handler::InputHandler::new(key_map);

    // create event_loop and window
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(window_width, window_height))
        .with_title("Highway Architect")
        .build(&event_loop)
        .unwrap();

    // Create handle to graphics card. Change line to use different gpu backend.
    let gfx = gfx_wgpu::GfxState::new(&window, window_width, window_height).await;

    let mut state = hw_architect::State::new(gfx, window_width, window_height);

    let mut last_render_time = Instant::now();
    event_loop
        .run(|event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);
            use input::Action;
            use input_handler::InputEvent;
            match input_handler.process_input(&event, window.id()) {
                InputEvent::KeyActions(actions) => {
                    if actions.contains(&(Action::Exit, input::KeyState::Press)) {
                        window_target.exit();
                    } else {
                        for a in actions {
                            // dbg!(a.clone());
                            state.key_input(a);
                        }
                    }
                }
                InputEvent::MouseEvent(e) => state.mouse_input(e),
                InputEvent::Absorb => {}
                InputEvent::Proceed => match event {
                    Event::WindowEvent { event, window_id } if window_id == window.id() => {
                        match event {
                            WindowEvent::CloseRequested => window_target.exit(),
                            WindowEvent::Resized(physical_size) => {
                                state.resize(physical_size.width, physical_size.height);
                            }
                            WindowEvent::RedrawRequested => {
                                window.request_redraw();

                                let now = Instant::now();

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
                                    Err(GfxFrameError::OutOfMemory) => window_target.exit(),
                                    // We're ignoring timeouts
                                    Err(GfxFrameError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
            }
        })
        .unwrap();
}
