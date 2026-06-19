mod config;
mod state;
mod protocols;
mod input;
mod render;
mod config_watcher;

use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use smithay::wayland::socket::ListeningSocketSource;
use smithay::backend::winit;
use state::State;
use input::KeyBindings;
use render::GlassRenderer;
use config_watcher::{load_config, spawn_config_watcher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo logger hệ thống
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Khởi chạy glass-wm (Wayland Window Manager)...");

    // 1. Tạo vòng lặp sự kiện calloop chính
    let mut event_loop = EventLoop::<State>::try_new()?;
    let loop_handle = event_loop.handle();

    // 2. Khởi tạo Wayland Display Server
    let display = Display::<State>::new()?;
    let display_handle = display.handle();

    // 3. Khởi tạo Trình theo dõi cấu hình (Hot-Reload TOML)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tdkhoa-01".to_string());
    let config_path = std::path::PathBuf::from(format!("{}/.config/glass-wm/config.toml", home));
    
    tracing::info!("Kiểm tra tệp cấu hình tại: {:?}", config_path);
    let initial_config = load_config(&config_path);
    
    // Khởi tạo trạng thái chính (State)
    let mut state = State::new(display_handle.clone(), loop_handle.clone());
    state.config = Arc::new(ArcSwap::new(Arc::new(initial_config)));

    // Bắt đầu background watcher
    let _watcher = spawn_config_watcher(config_path, state.config.clone());

    // 4. Mở cổng nghe socket Wayland client (ví dụ: "wayland-1")
    let listening_socket = ListeningSocketSource::new_auto()?;
    let socket_name = listening_socket.socket_name().to_string_lossy().into_owned();
    tracing::info!("Đang lắng nghe Wayland Client tại socket socket: {}", socket_name);

    // Đăng ký listening socket vào event loop
    loop_handle.insert_source(listening_socket, |client_stream, _, state| {
        // Chấp nhận client kết nối mới
        let _ = state.display_handle.insert_client(client_stream, Arc::new(state::ClientState {
            compositor_state: Default::default(),
        }));
    })?;

    // Đăng ký Wayland source xử lý client commands vào event loop
    let wayland_source = smithay::reexports::calloop::generic::Generic::new(
        display,
        smithay::reexports::calloop::Interest::READ,
        smithay::reexports::calloop::Mode::Level,
    );
    loop_handle.insert_source(wayland_source, move |_, display, state| {
        unsafe { display.get_mut() }.dispatch_clients(state)?;
        Ok(smithay::reexports::calloop::PostAction::Continue)
    })?;

    // Khởi tạo Keybindings
    let _keybindings = KeyBindings::new();

    // 5. Khởi chạy Winit Backend (Chạy lồng trong cửa sổ Desktop có sẵn để Test)
    let (mut backend, mut input) = winit::init::<smithay::backend::renderer::glow::GlowRenderer>()?;
    let glow_context = backend.renderer().with_context(|gl| gl.clone()).unwrap();
    let mut renderer = GlassRenderer::new(glow_context);
    renderer.resize(1024, 768);

    tracing::info!("Khởi động Winit backend thành công (1024x768).");

    // 6. Chạy vòng lặp Render/Event chính (Main loop)
    event_loop.run(
        Duration::from_millis(16), // ~60 FPS
        &mut state,
        move |state| {
            // Dispatch Wayland client buffers
            let _ = state.display_handle.flush_clients();

            // Nhận và phân phối sự kiện Input từ Winit
            let _ = input.dispatch_new_events(|event| {
                match event {
                    winit::WinitEvent::Resized { size, .. } => {
                        renderer.resize(size.w, size.h);
                    }
                    winit::WinitEvent::Input(_input_event) => {
                        // Nhận sự kiện phím ấn và kiểm tra keybindings
                        state.seat.get_keyboard().map(|_kbd| {
                            // Gọi keybindings.handle_key(modifiers, keycode, state)
                            // Ví dụ đơn giản: keybindings.handle_key(0, 36, state);
                        });
                    }
                    winit::WinitEvent::Redraw => {
                        let rect = smithay::utils::Rectangle::new((0, 0).into(), (1024, 768).into());
                        backend.bind().unwrap();
                        renderer.render_frame(state, rect);
                        backend.submit(None).unwrap();
                    }
                    _ => {}
                }
            });
        },
    )?;

    Ok(())
}
