mod auth;
mod ui;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::Local;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk4 as gtk;
use zeroize::Zeroizing;

fn get_username() -> String {
    std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            std::fs::read_dir("/home")
                .expect("cannot read /home")
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir() && e.file_name() != "lost+found")
                .min_by_key(|e| e.file_name())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .expect("no user directory found in /home")
        })
}

fn main() {
    std::panic::set_hook(Box::new(|info| eprintln!("PANIC: {info}")));
    let app = gtk::Application::new(None::<&str>, Default::default());
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let username = get_username();
    let ui::Widgets {
        window,
        hours,
        minutes,
        colon,
        date_label,
        dot,
    } = ui::build(app);

    setup_clock(&hours, &minutes, &colon, &date_label);
    setup_dot_pulse(&dot);

    let authenticating = Arc::new(AtomicBool::new(false));
    setup_keyboard(&window, &dot, &authenticating, &username);

    window.present();
}

fn setup_clock(
    hours: &gtk::Label,
    minutes: &gtk::Label,
    colon: &gtk::Label,
    date_label: &gtk::Label,
) {
    update_clock(hours, minutes, date_label);
    let colon_visible = Rc::new(Cell::new(true));

    glib::timeout_add_local(
        Duration::from_millis(500),
        clone!(
            #[weak]
            hours,
            #[weak]
            minutes,
            #[weak]
            colon,
            #[weak]
            date_label,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let vis = !colon_visible.get();
                colon_visible.set(vis);
                colon.set_opacity(if vis { 1.0 } else { 0.15 });
                if vis {
                    update_clock(&hours, &minutes, &date_label);
                }
                glib::ControlFlow::Continue
            }
        ),
    );
}

fn setup_dot_pulse(dot: &gtk::Box) {
    glib::timeout_add_local(
        Duration::from_millis(600),
        clone!(
            #[weak]
            dot,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                if dot.is_visible() {
                    if dot.has_css_class("bright") {
                        dot.remove_css_class("bright");
                    } else {
                        dot.add_css_class("bright");
                    }
                }
                glib::ControlFlow::Continue
            }
        ),
    );
}

fn setup_keyboard(
    window: &gtk::ApplicationWindow,
    dot: &gtk::Box,
    authenticating: &Arc<AtomicBool>,
    username: &str,
) {
    let password = Rc::new(RefCell::new(Zeroizing::new(String::new())));
    let auth_active = authenticating.clone();
    let key_ctrl = gtk::EventControllerKey::new();

    let username = Arc::new(username.to_owned());
    key_ctrl.connect_key_pressed(clone!(
        #[weak]
        dot,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, keyval, _, state| {
            if auth_active.load(Ordering::SeqCst) {
                return glib::Propagation::Stop;
            }
            match keyval {
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                    let pass = Zeroizing::new(password.borrow().to_string());
                    if pass.is_empty() {
                        return glib::Propagation::Stop;
                    }
                    password.borrow_mut().clear();
                    auth_active.store(true, Ordering::SeqCst);
                    dot.set_visible(true);

                    let auth_done = auth_active.clone();
                    let dot_weak = glib::SendWeakRef::from(dot.downgrade());
                    let username = username.clone();

                    std::thread::spawn(move || {
                        let result = auth::authenticate(&username, &pass);
                        auth_done.store(false, Ordering::SeqCst);
                        match result {
                            Ok(()) => std::process::exit(0),
                            Err(_) => {
                                glib::idle_add_once(move || {
                                    if let Some(dot) = dot_weak.upgrade() {
                                        dot.set_visible(false);
                                    }
                                });
                            }
                        }
                    });
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::BackSpace => {
                    password.borrow_mut().pop();
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Escape => {
                    password.borrow_mut().clear();
                    glib::Propagation::Stop
                }
                _ => {
                    if state.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                        || state.contains(gtk::gdk::ModifierType::ALT_MASK)
                    {
                        return glib::Propagation::Proceed;
                    }
                    if let Some(ch) = keyval.to_unicode()
                        && !ch.is_control()
                    {
                        password.borrow_mut().push(ch);
                    }
                    glib::Propagation::Stop
                }
            }
        }
    ));

    window.add_controller(key_ctrl);
}

fn update_clock(hours: &gtk::Label, minutes: &gtk::Label, date: &gtk::Label) {
    let now = Local::now();
    hours.set_text(&now.format("%H").to_string());
    minutes.set_text(&now.format("%M").to_string());
    date.set_text(&now.format("%b %d, %Y").to_string());
}
