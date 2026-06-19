use crate::state::State;
use smithay::wayland::compositor::{CompositorHandler, CompositorState, CompositorClientState};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::buffer::BufferHandler;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::delegate_compositor;
use smithay::delegate_shm;

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a smithay::reexports::wayland_server::Client) -> &'a CompositorClientState {
        &client.get_data::<crate::state::ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, _surface: &WlSurface) {
        // Xử lý khi client submit buffer mới cho surface (ví dụ: cập nhật hình học hoặc render damage)
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_compositor!(State);
delegate_shm!(State);
