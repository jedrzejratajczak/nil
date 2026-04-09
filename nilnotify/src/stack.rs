use std::sync::mpsc;

use crate::config::Colors;
use crate::notification::NotificationWindow;
use gtk4 as gtk;

const MAX_VISIBLE: usize = 4;
const GAP: i32 = 4;
const TOP_OFFSET: i32 = 4;

pub struct NotifyData {
    pub id: u32,
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub urgency: u8,
}

pub struct Stack {
    active: Vec<NotificationWindow>,
    queue: Vec<NotifyData>,
    next_id: u32,
    colors: Colors,
    dismiss_tx: mpsc::Sender<u32>,
}

impl Stack {
    pub fn new(colors: Colors, dismiss_tx: mpsc::Sender<u32>) -> Self {
        Self {
            active: Vec::new(),
            queue: Vec::new(),
            next_id: 1,
            colors,
            dismiss_tx,
        }
    }

    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        id
    }

    pub fn add(&mut self, app: &gtk::Application, data: NotifyData) {
        if self.active.len() >= MAX_VISIBLE {
            self.queue.push(data);
            return;
        }

        let margin_top = self.next_margin_top();
        let win = NotificationWindow::new(
            app,
            data.id,
            &data.summary,
            &data.body,
            &data.icon,
            data.urgency,
            &self.colors,
            margin_top,
            self.dismiss_tx.clone(),
        );
        self.active.push(win);
    }

    pub fn remove(&mut self, app: &gtk::Application, id: u32) {
        if let Some(idx) = self.active.iter().position(|n| n.id == id) {
            self.active[idx].close();
            self.active.remove(idx);
            self.reposition();

            if !self.queue.is_empty() && self.active.len() < MAX_VISIBLE {
                let data = self.queue.remove(0);
                self.add(app, data);
            }
        }
    }

    pub fn reload_colors(&mut self, colors: Colors) {
        self.colors = colors;
    }

    fn reposition(&self) {
        let mut y = TOP_OFFSET;
        for win in self.active.iter() {
            win.set_margin_top(y);
            y += win.height.get() + GAP;
        }
    }

    fn next_margin_top(&self) -> i32 {
        let mut y = TOP_OFFSET;
        for win in self.active.iter() {
            y += win.height.get() + GAP;
        }
        y
    }
}
