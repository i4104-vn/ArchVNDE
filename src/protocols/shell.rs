use crate::state::{State, Layer};
use smithay::wayland::shell::xdg::{
    XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState,
};
use smithay::wayland::shell::wlr_layer::{
    WlrLayerShellHandler, WlrLayerShellState, LayerSurface, Layer as WlrLayer,
};
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Rectangle, Point};
use smithay::desktop::{Window, layer_map_for_output, LayerSurface as DesktopLayerSurface};
use smithay::delegate_xdg_shell;
use smithay::delegate_layer_shell;

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.wayland.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface.clone());
        self.windows.space.map_element(window, Point::default(), true);
        let id = surface.wl_surface().id();
        self.add_window(id, Rectangle::new((50, 50).into(), (800, 600).into()), true, Layer::Top);
        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(
        &mut self,
        _surface: PopupSurface,
        _seat: wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {}

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {}
}

impl WlrLayerShellHandler for State {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.wayland.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        output: Option<WlOutput>,
        layer: WlrLayer,
        namespace: String,
    ) {
        let output = output
            .and_then(|o| smithay::output::Output::from_resource(&o))
            .unwrap_or_else(|| {
                self.windows
                    .space
                    .outputs()
                    .next()
                    .cloned()
                    .expect("no output available to map layer surface")
            });

        let desktop_surface = DesktopLayerSurface::new(surface, namespace);
        let mut layer_map = layer_map_for_output(&output);
        let _ = layer_map.map_layer(&desktop_surface);

        let custom_layer = match layer {
            WlrLayer::Background => Layer::Background,
            WlrLayer::Bottom => Layer::Bottom,
            WlrLayer::Top => Layer::Top,
            WlrLayer::Overlay => Layer::Overlay,
        };

        let id = desktop_surface.wl_surface().id();
        self.add_window(id, Rectangle::new((0, 0).into(), (1920, 48).into()), false, custom_layer);
    }
}

delegate_xdg_shell!(State);
delegate_layer_shell!(State);

mod wl_seat {
    pub use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
}
