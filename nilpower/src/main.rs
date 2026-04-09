use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;
use std::time::SystemTime;

use glib::ExitCode;
use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib};
use gtk4_layer_shell::LayerShell;

fn pixbuf_to_texture(pb: &gtk::gdk_pixbuf::Pixbuf) -> gdk::MemoryTexture {
    let format = if pb.has_alpha() {
        gdk::MemoryFormat::R8g8b8a8
    } else {
        gdk::MemoryFormat::R8g8b8
    };
    let bytes = glib::Bytes::from(unsafe { &*pb.pixels() });
    gdk::MemoryTexture::new(
        pb.width(),
        pb.height(),
        format,
        &bytes,
        pb.rowstride() as usize,
    )
}

const STYLE_CSS: &str = include_str!("style.css");

const ICON_SHUTDOWN: &[u8] = include_bytes!("icons/shutdown.png");
const ICON_REBOOT: &[u8] = include_bytes!("icons/reboot.png");
const ICON_LOCK: &[u8] = include_bytes!("icons/lock.png");

struct Action {
    label: &'static str,
    icon: &'static [u8],
    command: &'static [&'static str],
}

const ACTIONS: &[Action] = &[
    Action {
        label: "Shutdown",
        icon: ICON_SHUTDOWN,
        command: &["systemctl", "poweroff"],
    },
    Action {
        label: "Reboot",
        icon: ICON_REBOOT,
        command: &["systemctl", "reboot"],
    },
    Action {
        label: "Lock",
        icon: ICON_LOCK,
        command: &["hyprlock"],
    },
];

fn query_wallpaper() -> Option<String> {
    let output = Command::new("sh")
        .args(["-c", "awww query 2>/dev/null | grep -oP 'image: \\K.*'"])
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() || !std::path::Path::new(&path).is_file() {
        None
    } else {
        Some(path)
    }
}

fn execute_action(action: &Action) {
    let args = action.command;
    if args.len() == 1 {
        let _ = Command::new(args[0]).spawn();
    } else {
        let _ = Command::new(args[0]).args(&args[1..]).spawn();
    }
}

fn update_selection(buttons: &[gtk::Button], selected: usize) {
    for (i, button) in buttons.iter().enumerate() {
        if i == selected {
            button.add_css_class("selected");
        } else {
            button.remove_css_class("selected");
        }
    }
}

fn show_power_menu(app: &gtk::Application, window_ref: &Rc<RefCell<Option<gtk::ApplicationWindow>>>) {
    // If already open, close
    if let Some(ref win) = *window_ref.borrow() {
        if win.is_visible() {
            win.close();
            return;
        }
    }

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .build();

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.set_namespace(Some("nilpower"));
    window.set_exclusive_zone(-1);

    use gtk4_layer_shell::Edge;
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    let overlay = gtk::Overlay::new();

    // Wallpaper (supports GIF animation via PixbufAnimation)
    if let Some(path) = query_wallpaper() {
        let picture = gtk::Picture::new();
        picture.set_content_fit(gtk::ContentFit::Cover);
        picture.set_hexpand(true);
        picture.set_vexpand(true);

        match gtk::gdk_pixbuf::PixbufAnimation::from_file(&path) {
            Ok(anim) => {
                if anim.is_static_image() {
                    if let Some(pb) = anim.static_image() {
                        picture.set_paintable(Some(&pixbuf_to_texture(&pb)));
                    }
                } else {
                    let iter = anim.iter(None);
                    picture.set_paintable(Some(&pixbuf_to_texture(&iter.pixbuf())));
                    glib::timeout_add_local(
                        std::time::Duration::from_millis(30),
                        glib::clone!(
                            #[weak]
                            picture,
                            #[upgrade_or]
                            glib::ControlFlow::Break,
                            move || {
                                if iter.advance(SystemTime::now()) {
                                    picture.set_paintable(Some(&pixbuf_to_texture(&iter.pixbuf())));
                                }
                                glib::ControlFlow::Continue
                            }
                        ),
                    );
                }
            }
            Err(e) => {
                eprintln!("Warning: could not load wallpaper: {e}");
            }
        }

        overlay.set_child(Some(&picture));
    }

    // Dark overlay
    let dark = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .css_classes(["dark-overlay"])
        .build();
    overlay.add_overlay(&dark);

    // Button container
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .spacing(48)
        .build();

    let mut buttons: Vec<gtk::Button> = Vec::new();

    for action in ACTIONS {
        let bytes = glib::Bytes::from(action.icon);
        let texture = gdk::Texture::from_bytes(&bytes).expect("failed to load icon");
        let image = gtk::Image::from_paintable(Some(&texture));
        let label = gtk::Label::new(Some(action.label));

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .spacing(0)
            .build();
        content.append(&image);
        content.append(&label);

        let button = gtk::Button::builder()
            .child(&content)
            .css_classes(["power-button"])
            .focusable(false)
            .build();

        let cmd = action.command;
        let win = window.clone();
        button.connect_clicked(move |_| {
            execute_action(&ACTIONS[if cmd[0] == "systemctl" {
                if cmd[1] == "poweroff" { 0 } else { 1 }
            } else { 2 }]);
            win.close();
        });

        buttons.push(button.clone());
        container.append(&button);
    }

    overlay.add_overlay(&container);
    window.set_child(Some(&overlay));

    // Initial selection
    let selected = Rc::new(Cell::new(0usize));
    update_selection(&buttons, 0);

    // Hover selection
    for (i, button) in buttons.iter().enumerate() {
        let motion = gtk::EventControllerMotion::new();
        let sel = selected.clone();
        let btns = buttons.clone();
        motion.connect_enter(move |_, _, _| {
            sel.set(i);
            update_selection(&btns, i);
        });
        button.add_controller(motion);
    }

    // Keyboard
    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
    let win = window.clone();
    let sel = selected;
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        let current = sel.get();
        match key {
            gdk::Key::Escape => {
                win.close();
                glib::Propagation::Stop
            }
            gdk::Key::Return | gdk::Key::KP_Enter => {
                execute_action(&ACTIONS[current]);
                win.close();
                glib::Propagation::Stop
            }
            gdk::Key::s => {
                execute_action(&ACTIONS[0]);
                win.close();
                glib::Propagation::Stop
            }
            gdk::Key::r => {
                execute_action(&ACTIONS[1]);
                win.close();
                glib::Propagation::Stop
            }
            gdk::Key::l => {
                execute_action(&ACTIONS[2]);
                win.close();
                glib::Propagation::Stop
            }
            gdk::Key::Left | gdk::Key::h => {
                if current > 0 {
                    let new = current - 1;
                    sel.set(new);
                    update_selection(&buttons, new);
                }
                glib::Propagation::Stop
            }
            gdk::Key::Right => {
                if current < ACTIONS.len() - 1 {
                    let new = current + 1;
                    sel.set(new);
                    update_selection(&buttons, new);
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window.add_controller(key_ctrl);

    // Clear reference on close
    let wref = window_ref.clone();
    window.connect_close_request(move |_| {
        *wref.borrow_mut() = None;
        glib::Propagation::Proceed
    });

    *window_ref.borrow_mut() = Some(window.clone());
    window.present();
}

fn main() -> ExitCode {
    let daemon = std::env::args().any(|a| a == "--daemon");

    let app = gtk::Application::builder()
        .application_id("dev.mrozelek.nilpower")
        .build();

    let css_loaded = Rc::new(Cell::new(false));
    let window_ref: Rc<RefCell<Option<gtk::ApplicationWindow>>> = Rc::new(RefCell::new(None));
    let first_activate = Rc::new(Cell::new(true));

    let css_flag = css_loaded.clone();
    let wref = window_ref.clone();
    let first = first_activate.clone();
    let is_daemon = daemon;
    app.connect_activate(move |app| {
        // Load CSS once
        if !css_flag.get() {
            let display = gdk::Display::default().expect("no display");
            let css = gtk::CssProvider::new();
            css.load_from_string(STYLE_CSS);
            gtk::style_context_add_provider_for_display(
                &display,
                &css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            css_flag.set(true);
        }

        // Daemon: skip first activate (startup)
        if first.get() {
            first.set(false);
            if is_daemon {
                return;
            }
        }

        show_power_menu(app, &wref);
    });

    // Hold must live until app.run returns
    let _guard = if daemon { Some(app.hold()) } else { None };

    // Filter out --daemon so GTK doesn't complain
    let args: Vec<String> = std::env::args().filter(|a| a != "--daemon").collect();
    app.run_with_args(&args)
}
