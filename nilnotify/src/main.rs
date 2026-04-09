mod config;
mod dbus;
mod notification;
mod stack;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use glib::ExitCode;
use gtk4::prelude::*;
use gtk4::{self as gtk, glib};

use config::Colors;
use stack::{NotifyData, Stack};

fn main() -> ExitCode {
    let app = gtk::Application::builder()
        .application_id("dev.mrozelek.nilnotify")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &gtk::Application) {
    let colors = Colors::load();

    // Channel for dismiss events (notification -> stack)
    let (dismiss_tx, dismiss_rx) = mpsc::channel::<u32>();

    let stack = Rc::new(RefCell::new(Stack::new(colors, dismiss_tx.clone())));

    // D-Bus server channel (dbus thread -> gtk main thread)
    let (dbus_tx, dbus_rx) = mpsc::channel::<dbus::NotifyRequest>();
    dbus::run_server(dbus_tx);

    // Poll both channels from GTK main loop every 50ms
    let stack_clone = stack.clone();
    let app_clone = app.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        // Process dismiss events
        while let Ok(id) = dismiss_rx.try_recv() {
            stack_clone.borrow_mut().remove(&app_clone, id);
        }

        // Process new notification requests
        while let Ok(req) = dbus_rx.try_recv() {
            let mut s = stack_clone.borrow_mut();
            let id = s.next_id();
            s.add(
                &app_clone,
                NotifyData {
                    id,
                    summary: req.summary,
                    body: req.body,
                    icon: req.app_icon,
                    urgency: req.urgency,
                },
            );
        }

        glib::ControlFlow::Continue
    });

    // SIGUSR1 handler for color reload
    let sigusr1 = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&sigusr1))
        .expect("failed to register SIGUSR1 handler");
    let stack_clone2 = stack.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
        if sigusr1.swap(false, Ordering::Relaxed) {
            let new_colors = Colors::load();
            stack_clone2.borrow_mut().reload_colors(new_colors);
            eprintln!("nilnotify: colors reloaded");
        }
        glib::ControlFlow::Continue
    });

    // Keep app alive — leak the guard so it is never dropped (application runs until killed)
    std::mem::forget(app.hold());
}
