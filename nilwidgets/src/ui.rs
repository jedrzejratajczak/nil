use gtk4::prelude::*;
use gtk4::{self as gtk};

use crate::widgets;

pub struct WidgetHandles {
    pub panel: gtk::Box,
    pub cpu_label: gtk::Label,
    pub ram_label: gtk::Label,
    pub cpu_temp_label: gtk::Label,
    pub gpu_temp_label: gtk::Label,
    pub nvme_temp_label: gtk::Label,
    pub weather_icon: gtk::Label,
    pub weather_temp: gtk::Label,
    pub disk_label: gtk::Label,
    pub disk_bar_fill: gtk::Box,
    pub calendar_grid: gtk::Grid,
}

pub fn build() -> WidgetHandles {
    let panel = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(["widget-panel"])
        .valign(gtk::Align::Fill)
        .build();

    // CPU & RAM
    let (sysmon_box, cpu_label, ram_label) = widgets::sysmon::build();
    panel.append(&sysmon_box);
    panel.append(&separator());

    // Temperatures
    let (temps_box, cpu_temp_label, gpu_temp_label, nvme_temp_label) = widgets::temps::build();
    panel.append(&temps_box);
    panel.append(&separator());

    // Weather
    let (weather_box, weather_icon, weather_temp) = widgets::weather::build();
    panel.append(&weather_box);
    panel.append(&separator());

    // Disk
    let (disk_box, disk_label, disk_bar_fill) = widgets::disk::build();
    panel.append(&disk_box);
    panel.append(&separator());

    // Calendar
    let (cal_box, calendar_grid) = widgets::calendar::build();
    panel.append(&cal_box);

    WidgetHandles {
        panel,
        cpu_label,
        ram_label,
        cpu_temp_label,
        gpu_temp_label,
        nvme_temp_label,
        weather_icon,
        weather_temp,
        disk_label,
        disk_bar_fill,
        calendar_grid,
    }
}

fn separator() -> gtk::Box {
    gtk::Box::builder()
        .css_classes(["section-sep"])
        .build()
}
