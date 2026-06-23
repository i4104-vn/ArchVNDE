use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

pub struct NotificationWindow {
    pub window: gtk4::ApplicationWindow,
    pub title_label: gtk4::Label,
    pub body_label: gtk4::Label,
    pub icon_widget: gtk4::Image,
    pub active_timer: Rc<RefCell<Option<glib::SourceId>>>,
}

impl NotificationWindow {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::ApplicationWindow::new(app);
        window.init_layer_shell();

        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 15);
        window.set_margin(Edge::Right, 15);
        window.set_default_size(320, 80);
        window.add_css_class("notification-card");

        let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        box_layout.add_css_class("notification-box");
        box_layout.set_margin_start(12);
        box_layout.set_margin_end(12);
        box_layout.set_margin_top(12);
        box_layout.set_margin_bottom(12);

        let icon_widget = gtk4::Image::from_icon_name("dialog-information");
        icon_widget.set_pixel_size(36);
        box_layout.append(&icon_widget);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        
        let title_label = gtk4::Label::new(Some("System Notification"));
        title_label.set_xalign(0.0);
        title_label.add_css_class("notification-title");

        let body_label = gtk4::Label::new(Some("Welcome to ArchVNDE."));
        body_label.set_xalign(0.0);
        body_label.add_css_class("notification-body");

        text_box.append(&title_label);
        text_box.append(&body_label);
        box_layout.append(&text_box);

        window.set_child(Some(&box_layout));
        window.set_visible(false);

        Self {
            window,
            title_label,
            body_label,
            icon_widget,
            active_timer: Rc::new(RefCell::new(None)),
        }
    }

    pub fn update(&self, summary: &str, body: &str, icon: &str) {
        self.title_label.set_text(summary);
        self.body_label.set_text(body);

        if !icon.is_empty() {
            if icon.starts_with('/') {
                self.icon_widget.set_from_file(Some(icon));
            } else {
                self.icon_widget.set_icon_name(Some(icon));
            }
        } else {
            self.icon_widget.set_icon_name(Some("dialog-information"));
        }
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }
}
