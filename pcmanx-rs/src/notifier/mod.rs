#![allow(dead_code)]

use anyhow::Result;

pub struct Notifier {
    timeout_secs: u32,
    icon_path: Option<String>,
}

impl Notifier {
    pub fn new(timeout_secs: u32, icon_path: Option<String>) -> Self {
        Self { timeout_secs, icon_path }
    }

    /// Show a desktop popup notification.
    pub fn notify(&self, summary: &str, body: &str) -> Result<()> {
        let mut n = notify_rust::Notification::new();
        n.summary(summary)
            .body(body)
            .timeout(notify_rust::Timeout::Milliseconds(self.timeout_secs * 1000));
        if let Some(ref icon) = self.icon_path {
            n.icon(icon);
        }
        n.show()?;
        Ok(())
    }
}
