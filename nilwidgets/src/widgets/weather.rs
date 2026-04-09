use gtk4::prelude::*;
use gtk4::{self as gtk};

pub fn build() -> (gtk::Box, gtk::Label, gtk::Label) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();

    let title = gtk::Label::builder()
        .label("WEATHER")
        .css_classes(["section-title"])
        .halign(gtk::Align::Start)
        .build();

    let icon_label = gtk::Label::builder()
        .label("...")
        .css_classes(["weather-icon"])
        .halign(gtk::Align::Center)
        .build();

    let temp_label = gtk::Label::builder()
        .label("—")
        .css_classes(["weather-temp"])
        .halign(gtk::Align::Center)
        .build();

    container.append(&title);
    container.append(&icon_label);
    container.append(&temp_label);

    (container, icon_label, temp_label)
}
