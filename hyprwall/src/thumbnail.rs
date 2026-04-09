use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, gdk_pixbuf, glib};

pub const THUMB_WIDTH: i32 = 180;
pub const THUMB_HEIGHT: i32 = 108;

pub fn create_cell(path: &Path) -> (gtk::Box, gtk::Picture) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .css_classes(["wallpaper-cell"])
        .build();

    let thumb_frame = gtk::Box::builder()
        .width_request(THUMB_WIDTH)
        .height_request(THUMB_HEIGHT)
        .overflow(gtk::Overflow::Hidden)
        .hexpand(false)
        .vexpand(false)
        .css_classes(["wallpaper-thumb-frame"])
        .build();

    let picture = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Cover)
        .can_shrink(true)
        .width_request(THUMB_WIDTH)
        .height_request(THUMB_HEIGHT)
        .build();

    thumb_frame.append(&picture);

    let label = gtk::Label::builder()
        .label(path.file_name().unwrap_or_default().to_string_lossy().as_ref())
        .css_classes(["wallpaper-name"])
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .max_width_chars(24)
        .halign(gtk::Align::Center)
        .build();

    container.append(&thumb_frame);
    container.append(&label);

    (container, picture)
}

pub fn load_static(picture: &gtk::Picture, path: &Path) {
    match gdk::Texture::from_filename(path) {
        Ok(texture) => picture.set_paintable(Some(&texture)),
        Err(e) => eprintln!("failed to load {}: {}", path.display(), e),
    }
}

pub fn load_gif(picture: &gtk::Picture, path: &Path) -> Option<glib::SourceId> {
    let anim = match gdk_pixbuf::PixbufAnimation::from_file(path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("failed to load gif {}: {}", path.display(), e);
            return None;
        }
    };

    if anim.is_static_image() {
        if let Some(pixbuf) = anim.static_image() {
            let texture = gdk::Texture::for_pixbuf(&pixbuf);
            picture.set_paintable(Some(&texture));
        }
        return None;
    }

    let iter = anim.iter(Some(SystemTime::now()));
    let pixbuf = iter.pixbuf();
    let texture = gdk::Texture::for_pixbuf(&pixbuf);
    picture.set_paintable(Some(&texture));

    let iter = Rc::new(RefCell::new(iter));
    let picture = picture.clone();

    let source_id = glib::timeout_add_local(
        std::time::Duration::from_millis(50),
        move || {
            let iter_ref = iter.borrow_mut();
            if iter_ref.advance(SystemTime::now()) {
                let pixbuf = iter_ref.pixbuf();
                let texture = gdk::Texture::for_pixbuf(&pixbuf);
                picture.set_paintable(Some(&texture));
            }
            glib::ControlFlow::Continue
        },
    );

    Some(source_id)
}
