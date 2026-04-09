use gtk4::prelude::*;
use gtk4::{self as gtk};

pub fn build() -> (gtk::Box, gtk::Label, gtk::Box) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();

    let title = gtk::Label::builder()
        .label("DISK")
        .css_classes(["section-title"])
        .halign(gtk::Align::Start)
        .build();

    let label = gtk::Label::builder()
        .label("—")
        .css_classes(["sysmon-value"])
        .halign(gtk::Align::Start)
        .build();

    // Visual bar
    let bar_bg = gtk::Box::builder()
        .css_classes(["disk-bar"])
        .hexpand(true)
        .build();

    let bar_fill = gtk::Box::builder()
        .css_classes(["disk-bar-fill"])
        .build();
    bar_fill.set_size_request(0, -1);

    bar_bg.append(&bar_fill);

    container.append(&title);
    container.append(&label);
    container.append(&bar_bg);

    (container, label, bar_fill)
}
