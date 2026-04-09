use std::collections::HashMap;
use zbus::{interface, zvariant::OwnedValue, connection};

pub struct NotifyRequest {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub urgency: u8,
    pub expire_timeout: i32,
}

pub struct NotificationServer {
    tx: std::sync::mpsc::Sender<NotifyRequest>,
    next_id: std::sync::atomic::AtomicU32,
}

impl NotificationServer {
    pub fn new(tx: std::sync::mpsc::Sender<NotifyRequest>) -> Self {
        Self {
            tx,
            next_id: std::sync::atomic::AtomicU32::new(1),
        }
    }

    fn alloc_id(&self) -> u32 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .max(1)
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".into(), "icon-static".into()]
    }

    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id > 0 {
            replaces_id
        } else {
            self.alloc_id()
        };

        let urgency = hints
            .get("urgency")
            .and_then(|v| u8::try_from(v).ok())
            .unwrap_or(1);

        // Icon: prefer app_icon parameter, fallback to image-path hint
        let icon = if !app_icon.is_empty() {
            app_icon.to_string()
        } else {
            hints
                .get("image-path")
                .and_then(|v| <String as TryFrom<OwnedValue>>::try_from(v.clone()).ok())
                .unwrap_or_default()
        };

        let _ = self.tx.send(NotifyRequest {
            app_name: app_name.into(),
            replaces_id,
            app_icon: icon,
            summary: summary.into(),
            body: body.into(),
            urgency,
            expire_timeout,
        });

        id
    }

    fn close_notification(&self, _id: u32) {
        // No-op — notifications auto-expire
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "hyprnotify".into(),
            "mrozelek".into(),
            env!("CARGO_PKG_VERSION").into(),
            "1.2".into(),
        )
    }
}

pub fn run_server(tx: std::sync::mpsc::Sender<NotifyRequest>) {
    std::thread::spawn(move || {
        let rt = async {
            let server = NotificationServer::new(tx);
            let _conn = connection::Builder::session()
                .expect("session bus")
                .name("org.freedesktop.Notifications")
                .expect("bus name")
                .serve_at("/org/freedesktop/Notifications", server)
                .expect("serve")
                .build()
                .await
                .expect("build connection");

            std::future::pending::<()>().await;
        };
        async_io::block_on(rt);
    });
}
