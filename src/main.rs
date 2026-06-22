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
                BackendEvent::Redraw => {
                    if let Some((w, h)) = pending_resize.take() {
                        renderer.resize(w, h);
                    }
                    let rect = smithay::utils::Rectangle::new((0, 0).into(), current_size.into());
                    
                    let glow_renderer = winit_backend.renderer();
                    renderer.prepare(state, glow_renderer);

                    winit_backend.render_with(current_size.0, current_size.1, |frame| {
                        renderer.render_frame(state, rect, frame);
                    });
                    winit_backend.submit();
                }
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

                            if state_btn == ButtonState::Pressed && button == 272 {
                                let pos = pointer.current_location();
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
                                } else {
                                    let keyboard = state.input.seat.get_keyboard().unwrap();
                                    keyboard.set_focus(state, None, serial);
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
    })?;

    Ok(())
}
