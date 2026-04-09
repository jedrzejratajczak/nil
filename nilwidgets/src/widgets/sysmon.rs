use gtk4::prelude::*;
use gtk4::{self as gtk};

pub fn build() -> (gtk::Box, gtk::Label, gtk::Label) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();

    // CPU
    let cpu_title = gtk::Label::builder()
        .label("CPU")
        .css_classes(["section-title"])
        .halign(gtk::Align::Start)
        .build();
    let cpu_label = gtk::Label::builder()
        .label("—")
        .css_classes(["sysmon-value"])
        .halign(gtk::Align::Start)
        .build();

    // RAM
    let ram_title = gtk::Label::builder()
        .label("RAM")
        .css_classes(["section-title"])
        .halign(gtk::Align::Start)
        .build();
    ram_title.set_margin_top(12);
    let ram_label = gtk::Label::builder()
        .label("—")
        .css_classes(["sysmon-value"])
        .halign(gtk::Align::Start)
        .build();

    container.append(&cpu_title);
    container.append(&cpu_label);
    container.append(&ram_title);
    container.append(&ram_label);

    (container, cpu_label, ram_label)
}
