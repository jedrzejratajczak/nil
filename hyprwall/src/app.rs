use std::cell::{Cell, RefCell};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::rc::Rc;

use futures_channel::mpsc;
use futures_util::StreamExt;
use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib};
use gtk4_layer_shell::LayerShell;

use crate::thumbnail;
use crate::wallpaper;

const COLUMNS: u32 = 3;
const GREETER_BG: &str = "/usr/share/hyprgreeter/background.gif";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Target {
    Desktop,
    Greeter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Grid,
    TargetChooser,
    Applying,
}

#[derive(Debug)]
enum Msg {
    Toggle,
    Confirm(usize),
    ApplyTarget(Target),
    Back,
    ApplyFinished(bool),
}

type Tx = mpsc::UnboundedSender<Msg>;

struct State {
    wallpapers: Vec<PathBuf>,
    page: Page,
    page_shared: Rc<Cell<Page>>,
    selected_idx: usize,
    gif_sources: Rc<RefCell<Vec<glib::SourceId>>>,
    generation: Rc<Cell<u64>>,
    window: Option<gtk::Window>,
    flow_box: Option<gtk::FlowBox>,
    stack: Option<gtk::Stack>,
    btn_desktop: Option<gtk::Button>,
}

pub fn run() {
    let app = gtk::Application::builder()
        .application_id("dev.mrozelek.hyprwall")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(|app| {
        let display = gdk::Display::default().expect("no display");

        let colors_path = format!(
            "{}/.config/hyprwall/colors.css",
            std::env::var("HOME").unwrap_or_default()
        );
        if std::path::Path::new(&colors_path).exists() {
            let colors_css = gtk::CssProvider::new();
            colors_css.load_from_path(&colors_path);
            gtk::style_context_add_provider_for_display(
                &display,
                &colors_css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let css = gtk::CssProvider::new();
        css.load_from_data(include_str!("style.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        std::mem::forget(app.hold());

        let (tx, rx) = mpsc::unbounded::<Msg>();

        start_listener(tx.clone());

        let page_shared = Rc::new(Cell::new(Page::Grid));
        let mut state = State {
            wallpapers: Vec::new(),
            page: Page::Grid,
            page_shared: page_shared.clone(),
            selected_idx: 0,
            gif_sources: Rc::new(RefCell::new(Vec::new())),
            generation: Rc::new(Cell::new(0)),
            window: None,
            flow_box: None,
            stack: None,
            btn_desktop: None,
        };

        let app = app.clone();
        let tx_inner = tx.clone();

        glib::spawn_future_local(async move {
            let mut rx = rx;
            while let Some(msg) = rx.next().await {
                process_msg(&mut state, msg, &app, &tx_inner);
            }
        });
    });

    app.run();
}

fn process_msg(state: &mut State, msg: Msg, app: &gtk::Application, tx: &Tx) {
    match msg {
        Msg::Toggle => {
            if state.window.is_some() {
                hide(state);
            } else {
                show(state, app, tx);
            }
        }

        Msg::Back => match state.page {
            Page::TargetChooser => {
                state.page = Page::Grid;
                state.page_shared.set(Page::Grid);
                if let Some(stack) = &state.stack {
                    stack.set_visible_child_name("grid");
                }
            }
            _ => hide(state),
        },

        Msg::Confirm(idx) if state.page == Page::Grid => {
            state.selected_idx = idx;
            state.page = Page::TargetChooser;
            state.page_shared.set(Page::TargetChooser);
            if let Some(stack) = &state.stack {
                stack.set_visible_child_name("target");
            }
            if let Some(btn) = &state.btn_desktop {
                btn.grab_focus();
            }
        }

        Msg::ApplyTarget(target) if state.page == Page::TargetChooser => {
            state.page = Page::Applying;
            state.page_shared.set(Page::Applying);
            let path = state.wallpapers[state.selected_idx].clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let result = apply_wallpaper(&path, target);
                let _ = tx.unbounded_send(Msg::ApplyFinished(result));
            });
        }

        Msg::ApplyFinished(success) => {
            if success {
                hide(state);
            } else {
                state.page = Page::Grid;
                state.page_shared.set(Page::Grid);
                if let Some(stack) = &state.stack {
                    stack.set_visible_child_name("grid");
                }
            }
        }

        _ => {}
    }
}

fn show(state: &mut State, app: &gtk::Application, tx: &Tx) {
    let window = create_window(app);

    let flow_box = gtk::FlowBox::builder()
        .homogeneous(true)
        .max_children_per_line(COLUMNS)
        .min_children_per_line(3)
        .selection_mode(gtk::SelectionMode::Single)
        .row_spacing(8)
        .column_spacing(8)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Start)
        .build();

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .child(&flow_box)
        .build();

    let (target_page, btn_desktop) = build_target_chooser(tx);

    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(150)
        .build();
    stack.add_named(&scrolled, Some("grid"));
    stack.add_named(&target_page, Some("target"));
    stack.set_visible_child_name("grid");

    window.set_child(Some(&stack));

    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
    let tx_key = tx.clone();
    let fb = flow_box.clone();
    let page_ref = state.page_shared.clone();
    key_ctrl.connect_key_pressed(move |_, key, _, _| handle_key(key, &tx_key, &fb, &page_ref));
    window.add_controller(key_ctrl);

    let entries = wallpaper::discover_wallpapers();
    state.wallpapers = entries.iter().map(|e| e.path.clone()).collect();

    let current_gen = state.generation.get() + 1;
    state.generation.set(current_gen);

    for entry in &entries {
        let (cell, picture) = thumbnail::create_cell(&entry.path);
        flow_box.insert(&cell, -1);

        let pic = picture;
        let path = entry.path.clone();
        let is_gif = entry.is_gif;
        let sources = state.gif_sources.clone();
        let generation = state.generation.clone();

        glib::idle_add_local_once(move || {
            if generation.get() != current_gen {
                return;
            }
            if is_gif {
                if let Some(id) = thumbnail::load_gif(&pic, &path) {
                    sources.borrow_mut().push(id);
                }
            } else {
                thumbnail::load_static(&pic, &path);
            }
        });
    }

    let fb_focus = flow_box.clone();
    let win_ref = window.clone();
    window.connect_map(move |_| {
        if let Some(child) = fb_focus.child_at_index(0) {
            fb_focus.select_child(&child);
            child.set_focusable(true);
            gtk::prelude::GtkWindowExt::set_focus(&win_ref, Some(&child));
        }
    });

    window.present();

    state.window = Some(window);
    state.flow_box = Some(flow_box);
    state.stack = Some(stack);
    state.btn_desktop = Some(btn_desktop);
    state.page = Page::Grid;
}

fn hide(state: &mut State) {
    state.generation.set(state.generation.get() + 1);

    for id in state.gif_sources.borrow_mut().drain(..) {
        id.remove();
    }

    if let Some(window) = state.window.take() {
        window.destroy();
    }
    state.flow_box = None;
    state.stack = None;
    state.btn_desktop = None;
    state.wallpapers.clear();
    state.page = Page::Grid;
}

fn create_window(app: &gtk::Application) -> gtk::Window {
    let window = gtk::Window::builder()
        .application(app)
        .title("hyprwall")
        .build();

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.set_namespace(Some("hyprwall"));
    window.set_exclusive_zone(-1);

    use gtk4_layer_shell::Edge;
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    window
}

fn handle_key(key: gdk::Key, tx: &Tx, flow_box: &gtk::FlowBox, page: &Rc<Cell<Page>>) -> glib::Propagation {
    match key {
        gdk::Key::Escape => {
            let _ = tx.unbounded_send(Msg::Back);
            glib::Propagation::Stop
        }
        _ if page.get() == Page::TargetChooser => {
            glib::Propagation::Proceed
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
            let idx = flow_box
                .selected_children()
                .first()
                .map(|c| c.index() as usize)
                .unwrap_or(0);
            let _ = tx.unbounded_send(Msg::Confirm(idx));
            glib::Propagation::Stop
        }
        gdk::Key::h | gdk::Key::Left => {
            move_selection(flow_box, -1);
            glib::Propagation::Stop
        }
        gdk::Key::j | gdk::Key::Down => {
            move_selection(flow_box, COLUMNS as i32);
            glib::Propagation::Stop
        }
        gdk::Key::k | gdk::Key::Up => {
            move_selection(flow_box, -(COLUMNS as i32));
            glib::Propagation::Stop
        }
        gdk::Key::l | gdk::Key::Right => {
            move_selection(flow_box, 1);
            glib::Propagation::Stop
        }
        gdk::Key::d | gdk::Key::_1 => {
            let _ = tx.unbounded_send(Msg::ApplyTarget(Target::Desktop));
            glib::Propagation::Stop
        }
        gdk::Key::g | gdk::Key::_2 => {
            let _ = tx.unbounded_send(Msg::ApplyTarget(Target::Greeter));
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    }
}

fn build_target_chooser(tx: &Tx) -> (gtk::Box, gtk::Button) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(24)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let btn_desktop = gtk::Button::builder()
        .label("Desktop")
        .css_classes(["target-tile"])
        .focusable(true)
        .build();
    let btn_greeter = gtk::Button::builder()
        .label("Greeter")
        .css_classes(["target-tile"])
        .focusable(true)
        .build();

    let tx_d = tx.clone();
    btn_desktop.connect_clicked(move |_| {
        let _ = tx_d.unbounded_send(Msg::ApplyTarget(Target::Desktop));
    });
    let tx_g = tx.clone();
    btn_greeter.connect_clicked(move |_| {
        let _ = tx_g.unbounded_send(Msg::ApplyTarget(Target::Greeter));
    });

    container.append(&btn_desktop);
    container.append(&btn_greeter);

    (container, btn_desktop)
}

fn move_selection(flow_box: &gtk::FlowBox, offset: i32) {
    let current = flow_box
        .selected_children()
        .first()
        .map(|c| c.index())
        .unwrap_or(0);

    let new_idx = (current + offset).max(0);
    if let Some(child) = flow_box.child_at_index(new_idx) {
        flow_box.select_child(&child);
        child.grab_focus();
    }
}

fn apply_wallpaper(path: &PathBuf, target: Target) -> bool {
    match target {
        Target::Desktop => apply_desktop(path),
        Target::Greeter => apply_greeter(path),
    }
}

fn apply_desktop(path: &PathBuf) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let script = format!("{}/.config/awww/set-wallpaper.sh", home);

    match std::process::Command::new(&script)
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("set-wallpaper.sh failed: {}", stderr.trim());
            }
            output.status.success()
        }
        Err(e) => {
            eprintln!("failed to run set-wallpaper.sh: {}", e);
            false
        }
    }
}

fn apply_greeter(path: &PathBuf) -> bool {
    match std::process::Command::new("sudo")
        .args(["cp", &path.to_string_lossy(), GREETER_BG])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("greeter wallpaper copy failed: {}", stderr.trim());
            }
            output.status.success()
        }
        Err(e) => {
            eprintln!("failed to copy greeter wallpaper: {}", e);
            false
        }
    }
}

fn start_listener(tx: Tx) {
    let sock_path = crate::socket_path();
    let _ = std::fs::remove_file(&sock_path);
    let listener = UnixListener::bind(&sock_path).expect("cannot bind socket");

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if stream.is_ok() {
                let _ = tx.unbounded_send(Msg::Toggle);
            }
        }
    });
}
