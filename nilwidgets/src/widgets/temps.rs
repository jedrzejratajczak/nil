use gtk4::prelude::*;
use gtk4::{self as gtk};

pub fn build() -> (gtk::Box, gtk::Label, gtk::Label, gtk::Label) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();

    let title = gtk::Label::builder()
        .label("TEMPS")
        .css_classes(["section-title"])
        .halign(gtk::Align::Start)
        .build();
    container.append(&title);

    let cpu_temp = temp_row("CPU", &container);
    let gpu_temp = temp_row("GPU", &container);
    let nvme_temp = temp_row("NVMe", &container);

    (container, cpu_temp, gpu_temp, nvme_temp)
}

fn temp_row(name: &str, parent: &gtk::Box) -> gtk::Label {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let name_label = gtk::Label::builder()
        .label(name)
        .css_classes(["temp-name"])
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();

    let value_label = gtk::Label::builder()
        .label("—")
        .css_classes(["temp-value", "temp-ok"])
        .halign(gtk::Align::End)
        .build();

    row.append(&name_label);
    row.append(&value_label);
    parent.append(&row);

    value_label
}
