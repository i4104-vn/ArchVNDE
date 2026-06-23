use gtk4::prelude::*;
use gdk4::Texture;
use gdk_pixbuf::Pixbuf;

pub const WIFI_SVG: &str = include_str!("../assets/wifi.svg");
pub const BLUETOOTH_SVG: &str = include_str!("../assets/bluetooth.svg");
pub const PERFORMANCE_SVG: &str = include_str!("../assets/performance.svg");
pub const NIGHT_LIGHT_SVG: &str = include_str!("../assets/night-light.svg");
pub const DARK_MODE_SVG: &str = include_str!("../assets/dark-mode.svg");
pub const CAFFEINE_SVG: &str = include_str!("../assets/caffeine.svg");
pub const GSCONNECT_SVG: &str = include_str!("../assets/gsconnect.svg");
pub const PRIVACY_SVG: &str = include_str!("../assets/privacy.svg");
pub const SETTINGS_SVG: &str = include_str!("../assets/settings.svg");
pub const POWER_SVG: &str = include_str!("../assets/power.svg");
pub const VOLUME_SVG: &str = include_str!("../assets/volume.svg");
pub const BRIGHTNESS_SVG: &str = include_str!("../assets/brightness.svg");
pub const BATTERY_SVG: &str = include_str!("../assets/battery.svg");

/// Loads an SVG string into a GTK4 Image widget at a custom size.
pub fn get_icon_from_svg(svg_content: &str, size: i32) -> gtk4::Image {
    let bytes = glib::Bytes::from(svg_content.as_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    
    let pixbuf = Pixbuf::from_stream_at_scale(
        &stream,
        size,
        size,
        true,
        gio::Cancellable::NONE
    );

    match pixbuf {
        Ok(pb) => {
            let texture = Texture::for_pixbuf(&pb);
            gtk4::Image::from_paintable(Some(&texture))
        }
        Err(_) => {
            gtk4::Image::from_icon_name("image-missing")
        }
    }
}

/// Helper function to retrieve an SVG icon widget by name.
pub fn get_icon(name: &str, size: i32) -> gtk4::Image {
    let svg = match name {
        "wifi" => WIFI_SVG,
        "bluetooth" => BLUETOOTH_SVG,
        "performance" => PERFORMANCE_SVG,
        "night-light" => NIGHT_LIGHT_SVG,
        "dark-mode" => DARK_MODE_SVG,
        "caffeine" => CAFFEINE_SVG,
        "gsconnect" => GSCONNECT_SVG,
        "privacy" => PRIVACY_SVG,
        "settings" => SETTINGS_SVG,
        "power" => POWER_SVG,
        "volume" => VOLUME_SVG,
        "brightness" => BRIGHTNESS_SVG,
        "battery" => BATTERY_SVG,
        _ => return gtk4::Image::from_icon_name("image-missing"),
    };
    get_icon_from_svg(svg, size)
}
