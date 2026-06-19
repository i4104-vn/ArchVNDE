use smithay::backend::winit::{self, WinitGraphicsBackend, WinitInputBackend};

pub fn init() -> Result<(WinitGraphicsBackend, WinitInputBackend), Box<dyn std::error::Error>> {
    tracing::info!("Khởi tạo Winit Graphics và Input Backends...");
    let (backend, input) = winit::init()?;
    Ok((backend, input))
}
