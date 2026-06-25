pub mod easing;
pub mod fade;
pub mod slide;
pub mod zoom;

pub use fade::{fade_in, fade_out};
<<<<<<< HEAD:crates/archvnde-animation/src/lib.rs
pub use slide::{slide_in, SlideDirection};
=======
pub use slide::{slide_in, slide_out, SlideDirection};
pub use zoom::{zoom_in, zoom_out};
>>>>>>> e078269 (feat: implement smooth horizontal zoom-in/out transitions for system notch capsule):crates/archvnde-common/src/animation/mod.rs
