mod data;
mod ui;
mod widgets;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use glib::clone;
use glib::ExitCode;
use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib};
use gtk4_layer_shell::LayerShell;

const STYLE_CSS: &str = include_str!("style.css");

fn main() -> ExitCode {
    let app = gtk::Application::builder()
        .application_id("dev.mrozelek.nilwidgets")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &gtk::Application) {
    let display = gdk::Display::default().expect("no display");
    let css = gtk::CssProvider::new();
    css.load_from_string(STYLE_CSS);
    gtk::style_context_add_provider_for_display(
        &display,
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .build();

    // Layer shell setup
    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Bottom);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);
    window.set_namespace(Some("nilwidgets"));
    window.set_exclusive_zone(0);

    use gtk4_layer_shell::Edge;
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, false);

    // Build widget tree
    let handles = ui::build();
    window.set_child(Some(&handles.panel));

    // State
    let cpu_sampler = Rc::new(RefCell::new(data::cpu::CpuSampler::new()));
    let temp_reader = Rc::new(data::temperature::TempReader::discover());
    let current_date = Rc::new(RefCell::new(chrono::Local::now().date_naive()));

    // System timer (2s)
    glib::timeout_add_local(
        Duration::from_secs(2),
        clone!(
            #[weak(rename_to = cpu_label)]
            handles.cpu_label,
            #[weak(rename_to = ram_label)]
            handles.ram_label,
            #[weak(rename_to = cpu_temp_label)]
            handles.cpu_temp_label,
            #[weak(rename_to = gpu_temp_label)]
            handles.gpu_temp_label,
            #[weak(rename_to = nvme_temp_label)]
            handles.nvme_temp_label,
            #[weak(rename_to = disk_label)]
            handles.disk_label,
            #[weak(rename_to = disk_bar_fill)]
            handles.disk_bar_fill,
            #[weak(rename_to = calendar_grid)]
            handles.calendar_grid,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                // CPU
                let cpu_pct = cpu_sampler.borrow_mut().sample();
                cpu_label.set_text(&format!("{:.0}%", cpu_pct));

                // RAM
                let mem = data::memory::read_memory();
                ram_label.set_text(&format!("{:.1} / {:.1} GB", mem.used_gb, mem.total_gb));

                // Temperatures
                if let Some(ref reader) = *temp_reader {
                    let temps = reader.read();
                    update_temp_label(&cpu_temp_label, temps.cpu);
                    update_temp_label(&gpu_temp_label, temps.gpu);
                    update_temp_label(&nvme_temp_label, temps.nvme);
                }

                // Disk
                let disk = data::disk::read_disk();
                disk_label.set_text(&format!(
                    "{:.0} / {:.0} GB  ({}%)",
                    disk.used_gb, disk.total_gb, disk.percent as u32
                ));
                let pct = disk.percent.clamp(0.0, 100.0) as i32;
                disk_bar_fill.set_size_request(pct * 2, -1);

                // Calendar day change
                let today = chrono::Local::now().date_naive();
                if today != *current_date.borrow() {
                    *current_date.borrow_mut() = today;
                    widgets::calendar::rebuild(&calendar_grid);
                }

                glib::ControlFlow::Continue
            }
        ),
    );

    // Weather: initial fetch + 15min timer
    fetch_weather_async(&handles.weather_icon, &handles.weather_temp);
    glib::timeout_add_local(
        Duration::from_secs(15 * 60),
        clone!(
            #[weak(rename_to = weather_icon)]
            handles.weather_icon,
            #[weak(rename_to = weather_temp)]
            handles.weather_temp,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                fetch_weather_async(&weather_icon, &weather_temp);
                glib::ControlFlow::Continue
            }
        ),
    );

    window.present();
}

fn update_temp_label(label: &gtk::Label, temp: Option<f32>) {
    if let Some(t) = temp {
        label.set_text(&format!("{:.0}°C", t));
        label.remove_css_class("temp-ok");
        label.remove_css_class("temp-warm");
        label.remove_css_class("temp-hot");
        if t < 60.0 {
            label.add_css_class("temp-ok");
        } else if t < 80.0 {
            label.add_css_class("temp-warm");
        } else {
            label.add_css_class("temp-hot");
        }
    } else {
        label.set_text("N/A");
    }
}

fn fetch_weather_async(icon: &gtk::Label, temp: &gtk::Label) {
    let icon_ref = glib::SendWeakRef::from(icon.downgrade());
    let temp_ref = glib::SendWeakRef::from(temp.downgrade());

    std::thread::spawn(move || {
        match data::weather::fetch_weather() {
            Ok(weather) => {
                glib::idle_add_once(move || {
                    if let Some(icon) = icon_ref.upgrade() {
                        icon.set_text(data::weather::weather_icon(weather.weather_code));
                    }
                    if let Some(temp) = temp_ref.upgrade() {
                        temp.set_text(&format!("{:.0}°C", weather.temperature));
                    }
                });
            }
            Err(e) => eprintln!("Weather fetch failed: {e}"),
        }
    });
}
