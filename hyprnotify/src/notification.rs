use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;

use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib, pango};
use gtk4_layer_shell::LayerShell;

use crate::config::Colors;

const WIDTH: i32 = 280;
const MARGIN_RIGHT: i32 = 4;
const PADDING: i32 = 8;
const ICON_SIZE: i32 = 28;
const GAP_TO_TEXT: i32 = 8;

pub struct NotificationWindow {
    pub id: u32,
    pub window: gtk::Window,
    pub height: Cell<i32>,
    // Rc<Cell> so the timeout closure can clear it when it fires,
    // preventing close() from calling remove() on a dead SourceId.
    timeout_source: Rc<Cell<Option<glib::SourceId>>>,
}

impl NotificationWindow {
    pub fn new(
        app: &gtk::Application,
        id: u32,
        summary: &str,
        body: &str,
        icon_name: &str,
        urgency: u8,
        colors: &Colors,
        margin_top: i32,
        on_dismiss: mpsc::Sender<u32>,
    ) -> Self {
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .default_width(WIDTH)
            .resizable(false)
            .build();

        // Layer shell setup
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
        window.set_namespace(Some("hyprnotify"));

        use gtk4_layer_shell::Edge;
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, margin_top);
        window.set_margin(Edge::Right, MARGIN_RIGHT);

        // CSS styling
        let css = gtk::CssProvider::new();
        let css_str = format!(
            r#"
            window {{
                background-color: {};
                border-radius: 8px;
                padding: {}px;
            }}
            .summary {{
                font-size: 11px;
                font-weight: bold;
                color: {};
            }}
            .body {{
                font-size: 9px;
                color: alpha({}, 0.55);
            }}
            image {{
                -gtk-icon-filter: none;
                -gtk-icon-style: regular;
            }}
            "#,
            colors.surface, PADDING, colors.fg, colors.fg
        );
        css.load_from_string(&css_str);
        let display = gdk::Display::default().expect("no display");
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Layout: [icon] [text_box]
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, GAP_TO_TEXT);

        // Icon
        if !icon_name.is_empty() {
            let icon = gtk::Image::from_icon_name(icon_name);
            icon.set_pixel_size(ICON_SIZE);
            icon.set_size_request(ICON_SIZE, ICON_SIZE);
            icon.set_valign(gtk::Align::Center);
            hbox.append(&icon);
        }

        // Text
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 1);
        vbox.set_hexpand(true);

        let summary_label = gtk::Label::new(Some(summary));
        summary_label.set_xalign(0.0);
        summary_label.set_ellipsize(pango::EllipsizeMode::End);
        summary_label.set_max_width_chars(30);
        summary_label.add_css_class("summary");
        vbox.append(&summary_label);

        if !body.is_empty() {
            let body_label = gtk::Label::new(Some(body));
            body_label.set_xalign(0.0);
            body_label.set_ellipsize(pango::EllipsizeMode::End);
            body_label.set_max_width_chars(40);
            body_label.add_css_class("body");
            vbox.append(&body_label);
        }

        hbox.append(&vbox);
        window.set_child(Some(&hbox));

        // Right-click to dismiss
        let gesture = gtk::GestureClick::new();
        gesture.set_button(3);
        let dismiss_tx = on_dismiss.clone();
        let nid = id;
        gesture.connect_released(move |_, _, _, _| {
            let _ = dismiss_tx.send(nid);
        });
        window.add_controller(gesture);

        // Left-click to dismiss
        let gesture_left = gtk::GestureClick::new();
        gesture_left.set_button(1);
        let dismiss_tx2 = on_dismiss.clone();
        let nid2 = id;
        gesture_left.connect_released(move |_, _, _, _| {
            let _ = dismiss_tx2.send(nid2);
        });
        window.add_controller(gesture_left);

        // Timeout
        let timeout_ms: u64 = match urgency {
            0 => 3000,    // low
            2 => 0,       // critical — no auto-dismiss
            _ => 5000,    // normal
        };

        // Use Rc<Cell> so the timeout closure can clear the cell when it fires.
        // This prevents close() from calling remove() on an already-expired SourceId.
        let timeout_source: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

        if timeout_ms > 0 {
            let dismiss_tx3 = on_dismiss;
            let nid3 = id;
            let ts_clone = timeout_source.clone();
            let source_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(timeout_ms),
                move || {
                    // Clear the cell first so close() won't try to remove us
                    ts_clone.set(None);
                    let _ = dismiss_tx3.send(nid3);
                },
            );
            timeout_source.set(Some(source_id));
        }

        window.present();

        // Default height estimate — will be updated after map
        let height = Cell::new(if body.is_empty() { 30 } else { 45 });

        Self {
            id,
            window,
            height,
            timeout_source,
        }
    }

    pub fn set_margin_top(&self, margin: i32) {
        self.window.set_margin(gtk4_layer_shell::Edge::Top, margin);
    }

    pub fn close(&self) {
        if let Some(source) = self.timeout_source.take() {
            source.remove();
        }
        self.window.close();
    }
}
