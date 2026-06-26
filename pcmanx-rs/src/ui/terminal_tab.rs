#![allow(dead_code)]

use anyhow::Result;

use crate::config::SiteConfig;
use crate::connection::session::TelnetSession;
use crate::terminal::screen::ScreenBuffer;
use crate::ui::terminal_widget::{DisplaySettings, TerminalWidget};

/// One connection tab: a `TerminalWidget` backed by a live `TelnetSession`.
pub struct TerminalTab {
    pub widget: TerminalWidget,
    pub session: TelnetSession,
    pub title: String,
    pub connected: bool,
}

impl TerminalTab {
    /// Establish a new connection and build the tab.
    pub async fn connect(site: SiteConfig) -> Result<Self> {
        let display = DisplaySettings::default();
        let session = TelnetSession::new(site.clone()).await?;
        let screen = ScreenBuffer::new(
            site.rows_per_page as usize,
            site.cols_per_page as usize,
        );
        let title = site.name.clone();
        let widget = TerminalWidget::new(screen, display);

        Ok(Self {
            widget,
            session,
            title,
            connected: true,
        })
    }

    /// Read and process one batch of incoming data, updating the widget.
    pub async fn process_incoming(&mut self) -> Result<()> {
        self.session.process_incoming().await?;
        // Sync the rendered screen from the session.
        let screen = self.session.screen.clone();
        self.widget.set_screen(screen);
        Ok(())
    }

    /// Send raw bytes to the remote host (async version).
    pub async fn send_input(&mut self, data: &[u8]) -> Result<()> {
        self.session.send_key(data).await
    }
}
