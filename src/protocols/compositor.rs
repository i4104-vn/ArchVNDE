use crate::state::State;
use smithay::wayland::compositor::{CompositorHandler, CompositorState, CompositorClientState};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::seat::WaylandFocus;
use smithay::wayland::buffer::BufferHandler;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::delegate_compositor;
use smithay::delegate_shm;

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.wayland.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a smithay::reexports::wayland_server::Client,
    ) -> &'a CompositorClientState {
        &client.get_data::<crate::state::ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        tracing::debug!("commit surface: {:?}", surface);
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);

        if let Some(window) = self.windows.space.elements().find(|w| {
            w.wl_surface().as_ref().map(|cow| &**cow == surface).unwrap_or(false)
        }) {
            tracing::info!("commit: found window in space, calling window.on_commit()");
            window.on_commit();
        } else {
            // Check if it's a subsurface or popup
            tracing::debug!("commit: surface {:?} not directly mapped as window", surface);
        }
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.wayland.shm_state
    }
}

delegate_compositor!(State);
delegate_shm!(State);
