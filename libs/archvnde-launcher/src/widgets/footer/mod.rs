use gtk4::prelude::*;

mod render;

pub fn create_launcher_footer() -> gtk4::Box {
    let (
        footer_box,
        power_btn,
        power_popover,
        shutdown_btn,
        reboot_btn,
        suspend_btn,
    ) = render::build_footer_layout();

    shutdown_btn.connect_clicked(|_| {
        archvnde_common::poweroff();
    });

    reboot_btn.connect_clicked(|_| {
        archvnde_common::reboot();
    });

    suspend_btn.connect_clicked(|_| {
        archvnde_common::suspend();
    });

    let power_popover_clone = power_popover.clone();
    power_btn.connect_clicked(move |_| {
        power_popover_clone.popup();
    });

    footer_box
}
