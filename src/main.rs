mod config;
mod state;
mod protocols;
mod render;
mod backend;
mod config_watcher;

use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use smithay::wayland::socket::ListeningSocketSource;
use smithay::backend::input::{
    InputEvent, KeyboardKeyEvent, PointerButtonEvent, ButtonState, Event, AbsolutePositionEvent,
};

use state::State;
use render::GlassRenderer;
use backend::{CompositorBackend, BackendEvent};
use backend::winit::WinitBackend;
use config_watcher::{load_config, spawn_config_watcher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("starting ArchVNDE compositor");

    let mut event_loop = EventLoop::<State>::try_new()?;
    let loop_handle = event_loop.handle();

    let display = Display::<State>::new()?;
    let display_handle = display.handle();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tdkhoa-01".to_string());
    let config_path = std::path::PathBuf::from(format!("{}/.config/glass-wm/config.toml", home));

    let mut state = State::new(display_handle.clone(), loop_handle.clone());
    state.config = Arc::new(ArcSwap::new(Arc::new(load_config(&config_path))));
    let _watcher = spawn_config_watcher(config_path, state.config.clone());

    let socket = ListeningSocketSource::new_auto()?;
    tracing::info!("wayland socket: {}", socket.socket_name().to_string_lossy());

    loop_handle.insert_source(socket, |stream, _, state| {
        let _ = state.display_handle.insert_client(
            stream,
            Arc::new(state::ClientState { compositor_state: Default::default() }),
        );
    })?;

    loop_handle.insert_source(
        smithay::reexports::calloop::generic::Generic::new(
            display,
            smithay::reexports::calloop::Interest::READ,
            smithay::reexports::calloop::Mode::Level,
        ),
        move |_, display, state| {
            unsafe { display.get_mut() }.dispatch_clients(state)?;
            Ok(smithay::reexports::calloop::PostAction::Continue)
        },
    )?;

    let (mut winit_backend, gl_ctx) = WinitBackend::init()?;
    let mut renderer = GlassRenderer::new(gl_ctx);

    let (init_w, init_h) = winit_backend.current_size();
    renderer.resize(init_w as i32, init_h as i32);
    tracing::info!("winit backend ready ({}x{})", init_w, init_h);

    let mut current_size = (init_w as i32, init_h as i32);
    let mut pending_resize: Option<(i32, i32)> = None;

    event_loop.run(Duration::from_millis(16), &mut state, move |state| {
        let _ = state.display_handle.flush_clients();

        let mut events = Vec::new();
        winit_backend.poll_events(|e| events.push(e));

        for event in events {
            match event {
                BackendEvent::Resized { width, height } => {
                    pending_resize = Some((width as i32, height as i32));
                    current_size = (width as i32, height as i32);
                }
                BackendEvent::Redraw => {}
                BackendEvent::Input(input_event) => {
                    match input_event {
                        InputEvent::Keyboard { event } => {
                            let keyboard = state.input.seat.get_keyboard().unwrap();
                            let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                            let time = event.time_msec();
                            keyboard.input(
                                state,
                                event.key_code(),
                                event.state(),
                                serial,
                                time,
                                |state, modifiers, handle| {
                                    let keysym = handle.modified_sym();
                                    if modifiers.logo && keysym == smithay::input::keyboard::keysyms::KEY_Return.into() {
                                        let config = state.config.load();
                                        let _ = std::process::Command::new(&*config.shortcut.launch_terminal).spawn();
                                        smithay::input::keyboard::FilterResult::Intercept(())
                                    } else {
                                        smithay::input::keyboard::FilterResult::Forward
                                    }
                                },
                            );
                        }
                        InputEvent::PointerMotionAbsolute { event } => {
                            let pointer = state.input.seat.get_pointer().unwrap();
                            let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                            let time = event.time_msec();
                            let pos = event.position_transformed(smithay::utils::Size::from(current_size));

                            // Update window position if dragging
                            if let crate::state::window::PointerGrab::Move { ref window, start_cursor_pos, start_window_pos } = state.windows.pointer_grab {
                                let delta = pos - start_cursor_pos;
                                let new_pos = start_window_pos + smithay::utils::Point::from((delta.x as i32, delta.y as i32));
                                state.windows.space.map_element(window.clone(), new_pos, true);
                            }

                            let under = state.windows.space.element_under(pos).and_then(|(window, rel_pos)| {
                                window.surface_under(
                                    smithay::utils::Point::<f64, smithay::utils::Logical>::from((rel_pos.x as f64, rel_pos.y as f64)),
                                    smithay::desktop::WindowSurfaceType::all(),
                                )
                            }).map(|(surface, surface_rel_pos)| {
                                (surface, (surface_rel_pos.x as f64, surface_rel_pos.y as f64).into())
                            });
                            pointer.motion(
                                state,
                                under,
                                &smithay::input::pointer::MotionEvent {
                                    location: pos,
                                    serial,
                                    time,
                                },
                            );
                            pointer.frame(state);
                        }
                        InputEvent::PointerButton { event } => {
                            let pointer = state.input.seat.get_pointer().unwrap();
                            let serial = smithay::utils::SERIAL_COUNTER.next_serial();
                            let time = event.time_msec();
                            let button = event.button_code();
                            let state_btn = event.state();

                            pointer.button(
                                state,
                                &smithay::input::pointer::ButtonEvent {
                                    button,
                                    state: state_btn,
                                    serial,
                                    time,
                                },
                            );
                            pointer.frame(state);

                            if state_btn == ButtonState::Released && button == 272 {
                                state.windows.pointer_grab = crate::state::window::PointerGrab::None;
                            }

                            if state_btn == ButtonState::Pressed && button == 272 {
                                let pos = pointer.current_location();
                                let screen_w = current_size.0 as f64;
                                let screen_h = current_size.1 as f64;

                                // Dock geometry
                                let dock_w = 400.0;
                                let dock_h = 48.0;
                                let dock_x = (screen_w - dock_w) / 2.0;
                                let dock_y = screen_h - 60.0;

                                let in_dock = pos.x >= dock_x
                                    && pos.x <= dock_x + dock_w
                                    && pos.y >= dock_y
                                    && pos.y <= dock_y + dock_h;

                                if in_dock {
                                    let rel_x = pos.x - dock_x;
                                    if rel_x >= 40.0 && rel_x <= 72.0 {
                                        // Terminal
                                        let config = state.config.load();
                                        let _ = std::process::Command::new(&*config.shortcut.launch_terminal).spawn();
                                    } else if rel_x >= 130.0 && rel_x <= 162.0 {
                                        // Browser
                                        if std::process::Command::new("firefox").spawn().is_err() {
                                            if std::process::Command::new("chromium-browser").spawn().is_err() {
                                                let _ = std::process::Command::new("google-chrome").spawn();
                                            }
                                        }
                                    } else if rel_x >= 220.0 && rel_x <= 252.0 {
                                        // Files
                                        if std::process::Command::new("nautilus").spawn().is_err() {
                                            if std::process::Command::new("thunar").spawn().is_err() {
                                                let _ = std::process::Command::new("pcmanfm").spawn();
                                            }
                                        }
                                    } else if rel_x >= 310.0 && rel_x <= 342.0 {
                                        // Settings
                                        if std::process::Command::new("gnome-control-center").spawn().is_err() {
                                            let config = state.config.load();
                                            let _ = std::process::Command::new(&*config.shortcut.launch_terminal)
                                                .arg("-e")
                                                .arg("htop")
                                                .spawn();
                                        }
                                    }
                                } else {
                                    let window = state.windows.space.element_under(pos).map(|(w, _)| w.clone());
                                    if let Some(window) = window {
                                        state.windows.space.raise_element(&window, true);
                                        if let Some(toplevel) = window.toplevel() {
                                            toplevel.send_configure();
                                        }
                                        let keyboard = state.input.seat.get_keyboard().unwrap();
                                        keyboard.set_focus(
                                            state,
                                            Some(window.toplevel().unwrap().wl_surface().clone()),
                                            serial,
                                        );

                                        // Try starting a window drag grab
                                        let modifiers = keyboard.modifier_state();
                                        let is_header_click = if let Some(bbox) = state.windows.space.element_bbox(&window) {
                                            let rel_y = pos.y - bbox.loc.y as f64;
                                            rel_y >= 0.0 && rel_y <= 32.0
                                        } else {
                                            false
                                        };

                                        if modifiers.logo || is_header_click {
                                            let start_window_pos = state.windows.space.element_location(&window).unwrap_or_default();
                                            state.windows.pointer_grab = crate::state::window::PointerGrab::Move {
                                                window: window.clone(),
                                                start_cursor_pos: pos,
                                                start_window_pos,
                                            };
                                        }
                                    } else {
                                        let keyboard = state.input.seat.get_keyboard().unwrap();
                                        keyboard.set_focus(state, None, serial);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                BackendEvent::CloseRequested => {
                    tracing::info!("close requested — exiting");
                    std::process::exit(0);
                }
            }
        }

        // Redraw on every event loop iteration (60 FPS) to ensure smooth animations, cursor, and client updates
        {
            if let Some((w, h)) = pending_resize.take() {
                renderer.resize(w, h);
            }
            let rect = smithay::utils::Rectangle::new((0, 0).into(), current_size.into());
            
            state.windows.space.refresh();

            let glow_renderer = winit_backend.renderer();
            renderer.prepare(state, glow_renderer);

            winit_backend.render_with(current_size.0, current_size.1, |frame| {
                renderer.render_frame(state, rect, frame);
            });
            winit_backend.submit();
        }
    })?;

    Ok(())
}
