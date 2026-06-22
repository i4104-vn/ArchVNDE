mod config;
mod state;
mod protocols;
mod input;
mod render;
mod backend;
mod config_watcher;

use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use smithay::wayland::socket::ListeningSocketSource;

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
                    winit_backend.render_with(current_size.0, current_size.1, || {
                        renderer.render_frame(state, rect);
                    });
                    winit_backend.submit();
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
