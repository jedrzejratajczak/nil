use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk4 as gtk;
use std::time::{Duration, SystemTime};

const GIF_PATH: &str = "/usr/share/hyprgreeter/background.gif";

pub struct Widgets {
    pub window: gtk::ApplicationWindow,
    pub hours: gtk::Label,
    pub minutes: gtk::Label,
    pub colon: gtk::Label,
    pub date_label: gtk::Label,
    pub dot: gtk::Box,
}

pub fn build(app: &gtk::Application) -> Widgets {
    load_css();

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .build();

    let overlay = gtk::Overlay::new();
    window.set_child(Some(&overlay));

    setup_background(&overlay);

    let (card_overlay, hours, colon, minutes, date_label, dot) = build_card();

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.set_halign(gtk::Align::Center);
    container.set_valign(gtk::Align::Center);
    container.append(&card_overlay);
    overlay.add_overlay(&container);

    Widgets {
        window,
        hours,
        minutes,
        colon,
        date_label,
        dot,
    }
}

fn load_css() {
    let css = gtk::CssProvider::new();
    css.load_from_string(include_str!("style.css"));
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("No display"),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_background(overlay: &gtk::Overlay) {
    let picture = gtk::Picture::new();
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    match gtk::gdk_pixbuf::PixbufAnimation::from_file(GIF_PATH) {
        Ok(anim) => {
            if anim.is_static_image() {
                if let Some(pb) = anim.static_image() {
                    picture.set_paintable(Some(&pixbuf_to_texture(&pb)));
                }
            } else {
                let iter = anim.iter(None);
                picture.set_paintable(Some(&pixbuf_to_texture(&iter.pixbuf())));
                glib::timeout_add_local(
                    Duration::from_millis(30),
                    clone!(
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
            eprintln!("Warning: Could not load background GIF ({GIF_PATH}): {e}, using black background");
        }
    }

    overlay.set_child(Some(&picture));
}

fn build_card() -> (gtk::Overlay, gtk::Label, gtk::Label, gtk::Label, gtk::Label, gtk::Box) {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");

    let clock_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    clock_box.set_halign(gtk::Align::Center);

    let hours = gtk::Label::new(Some("00"));
    hours.add_css_class("clock");
    let colon = gtk::Label::new(Some(":"));
    colon.add_css_class("clock");
    let minutes = gtk::Label::new(Some("00"));
    minutes.add_css_class("clock");

    clock_box.append(&hours);
    clock_box.append(&colon);
    clock_box.append(&minutes);
    card.append(&clock_box);

    let sep = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    sep.add_css_class("separator");
    sep.set_halign(gtk::Align::Center);
    card.append(&sep);

    let date_label = gtk::Label::new(Some(""));
    date_label.add_css_class("date");
    card.append(&date_label);

    // Dot as absolute-positioned overlay inside the card
    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("dot");
    dot.set_halign(gtk::Align::End);
    dot.set_valign(gtk::Align::Start);
    dot.set_margin_top(16);
    dot.set_margin_end(16);
    dot.set_visible(false);

    let card_overlay = gtk::Overlay::new();
    card_overlay.set_child(Some(&card));
    card_overlay.add_overlay(&dot);

    (card_overlay, hours, colon, minutes, date_label, dot)
}

fn pixbuf_to_texture(pb: &gtk::gdk_pixbuf::Pixbuf) -> gtk::gdk::MemoryTexture {
    let format = if pb.has_alpha() {
        gtk::gdk::MemoryFormat::R8g8b8a8
    } else {
        gtk::gdk::MemoryFormat::R8g8b8
    };
    let bytes = glib::Bytes::from(unsafe { &*pb.pixels() });
    gtk::gdk::MemoryTexture::new(
        pb.width(),
        pb.height(),
        format,
        &bytes,
        pb.rowstride() as usize,
    )
}
