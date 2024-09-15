use crate::{widgets::window::Window, window_initiator::WindowInitiator};

use gtk::gdk;
pub use platform_wayland::WaylandBackend;

pub trait DisplayBackend: Send + Sync + 'static {
    fn initialize_window(window_init: &WindowInitiator, monitor: gdk::Rectangle, x: i32, y: i32) -> Option<Window>;
}

pub struct NoBackend;

impl DisplayBackend for NoBackend {
    fn initialize_window(_window_init: &WindowInitiator, _monitor: gdk::Rectangle, x: i32, y: i32) -> Option<Window> {
        Some(Window::new(gtk::WindowType::Toplevel, x, y))
    }
}

mod platform_wayland {
    use crate::{widgets::window::Window, window_initiator::WindowInitiator};
    use gtk::gdk;
    use gtk::prelude::*;
    use gtk_layer_shell::LayerShell;
    use yuck::config::{window_definition::WindowStacking, window_geometry::AnchorAlignment};

    use super::DisplayBackend;

    pub struct WaylandBackend;

    impl DisplayBackend for WaylandBackend {
        fn initialize_window(window_init: &WindowInitiator, monitor: gdk::Rectangle, x: i32, y: i32) -> Option<Window> {
            let window = Window::new(gtk::WindowType::Toplevel, x, y);
            // Initialising a layer shell surface
            window.init_layer_shell();
            // Sets the monitor where the surface is shown
            if let Some(ident) = window_init.monitor.clone() {
                let display = gdk::Display::default().expect("could not get default display");
                if let Some(monitor) = crate::app::get_monitor_from_display(&display, &ident) {
                    window.set_monitor(&monitor);
                } else {
                    return None;
                }
            };
            window.set_resizable(window_init.resizable);

            // Sets the layer where the layer shell surface will spawn
            match window_init.stacking {
                WindowStacking::Foreground => window.set_layer(gtk_layer_shell::Layer::Top),
                WindowStacking::Background => window.set_layer(gtk_layer_shell::Layer::Background),
                WindowStacking::Bottom => window.set_layer(gtk_layer_shell::Layer::Bottom),
                WindowStacking::Overlay => window.set_layer(gtk_layer_shell::Layer::Overlay),
            }

            if let Some(namespace) = &window_init.backend_options.wayland.namespace {
                window.set_namespace(namespace);
            }

            // Sets the keyboard interactivity
            window.set_keyboard_interactivity(window_init.backend_options.wayland.focusable);

            if let Some(geometry) = window_init.geometry {
                // Positioning surface
                let mut top = false;
                let mut left = false;
                let mut right = false;
                let mut bottom = false;

                match geometry.anchor_point.x {
                    AnchorAlignment::START => left = true,
                    AnchorAlignment::CENTER => {}
                    AnchorAlignment::END => right = true,
                }
                match geometry.anchor_point.y {
                    AnchorAlignment::START => top = true,
                    AnchorAlignment::CENTER => {}
                    AnchorAlignment::END => bottom = true,
                }

                window.set_anchor(gtk_layer_shell::Edge::Left, left);
                window.set_anchor(gtk_layer_shell::Edge::Right, right);
                window.set_anchor(gtk_layer_shell::Edge::Top, top);
                window.set_anchor(gtk_layer_shell::Edge::Bottom, bottom);

                let xoffset = geometry.offset.x.pixels_relative_to(monitor.width());
                let yoffset = geometry.offset.y.pixels_relative_to(monitor.height());

                if left {
                    window.set_layer_shell_margin(gtk_layer_shell::Edge::Left, xoffset);
                } else {
                    window.set_layer_shell_margin(gtk_layer_shell::Edge::Right, xoffset);
                }
                if bottom {
                    window.set_layer_shell_margin(gtk_layer_shell::Edge::Bottom, yoffset);
                } else {
                    window.set_layer_shell_margin(gtk_layer_shell::Edge::Top, yoffset);
                }
            }
            if window_init.backend_options.wayland.exclusive {
                window.auto_exclusive_zone_enable();
            }
            Some(window)
        }
    }
}
