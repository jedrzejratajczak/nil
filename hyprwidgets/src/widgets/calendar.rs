use chrono::{Datelike, Local, NaiveDate, Weekday};
use gtk4::prelude::*;
use gtk4::{self as gtk};

pub fn build() -> (gtk::Box, gtk::Grid) {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();

    let title = gtk::Label::builder()
        .label("CALENDAR")
        .css_classes(["section-title"])
        .halign(gtk::Align::Start)
        .build();

    let month_label = gtk::Label::builder()
        .css_classes(["cal-month"])
        .halign(gtk::Align::Center)
        .build();

    let grid = gtk::Grid::builder()
        .column_homogeneous(true)
        .row_homogeneous(true)
        .halign(gtk::Align::Center)
        .build();

    container.append(&title);
    container.append(&month_label);
    container.append(&grid);

    populate_grid(&grid, &month_label);

    (container, grid)
}

pub fn rebuild(grid: &gtk::Grid) {
    // Find month label (sibling before grid)
    if let Some(parent) = grid.parent() {
        let parent_box = parent.downcast_ref::<gtk::Box>().unwrap();
        // Month label is the second child (index 1)
        if let Some(child) = parent_box.first_child().and_then(|c| c.next_sibling()) {
            if let Some(month_label) = child.downcast_ref::<gtk::Label>() {
                // Clear grid
                while let Some(child) = grid.first_child() {
                    grid.remove(&child);
                }
                populate_grid(grid, month_label);
            }
        }
    }
}

fn populate_grid(grid: &gtk::Grid, month_label: &gtk::Label) {
    let today = Local::now().date_naive();
    let year = today.year();
    let month = today.month();

    month_label.set_text(&today.format("%B %Y").to_string());

    // Day headers
    let days = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    for (col, day) in days.iter().enumerate() {
        let label = gtk::Label::builder()
            .label(*day)
            .css_classes(["cal-header"])
            .build();
        grid.attach(&label, col as i32, 0, 1, 1);
    }

    // First day of month
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let start_weekday = first.weekday().num_days_from_monday() as i32;

    // Days in month
    let total_days = month_length(year, month);

    // Previous month days
    let prev_month_days = if month == 1 {
        month_length(year - 1, 12)
    } else {
        month_length(year, month - 1)
    };

    for i in 0..start_weekday {
        let day = prev_month_days - (start_weekday - 1 - i) as u32;
        let label = gtk::Label::builder()
            .label(&day.to_string())
            .css_classes(["cal-day", "cal-other-month"])
            .build();
        grid.attach(&label, i, 1, 1, 1);
    }

    // Current month days
    let mut col = start_weekday;
    let mut row = 1;
    for day in 1..=total_days {
        let mut classes = vec!["cal-day"];
        if day == today.day() {
            classes.push("cal-today");
        }
        let weekday = Weekday::try_from((col as u32 % 7) as u8).unwrap_or(Weekday::Mon);
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            classes.push("cal-weekend");
        }

        let label = gtk::Label::builder()
            .label(&day.to_string())
            .css_classes(classes)
            .build();
        grid.attach(&label, col, row, 1, 1);

        col += 1;
        if col >= 7 {
            col = 0;
            row += 1;
        }
    }

    // Next month days to fill the row
    if col > 0 {
        let mut next_day = 1u32;
        while col < 7 {
            let label = gtk::Label::builder()
                .label(&next_day.to_string())
                .css_classes(["cal-day", "cal-other-month"])
                .build();
            grid.attach(&label, col, row, 1, 1);
            col += 1;
            next_day += 1;
        }
    }
}

fn month_length(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}
