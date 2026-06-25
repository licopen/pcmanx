#![allow(dead_code)]

use anyhow::Result;
use vte::Parser;

use crate::charset::decode;
use crate::config::SiteConfig;
use crate::connection::telnet::TelnetConnection;
use crate::terminal::screen::ScreenBuffer;
use crate::terminal::vt100::Vt100Performer;

pub struct TelnetSession {
    pub connection: TelnetConnection,
    pub screen: ScreenBuffer,
    pub site: SiteConfig,
    parser: Parser,
}

impl TelnetSession {
    pub async fn new(site: SiteConfig) -> Result<Self> {
        let (host, port) = if let Some((host, port)) = site.url.rsplit_once(':') {
            let parsed_port = port
                .parse::<u16>()
                .map_err(|_| anyhow::anyhow!("Invalid port in site URL: {}", site.url))?;
            (host.to_string(), parsed_port)
        } else {
            (site.url.clone(), 23)
        };

        let mut connection = TelnetConnection::connect(&host, port).await?;
        connection.set_window_size(site.cols_per_page as u16, site.rows_per_page as u16);

        Ok(Self {
            connection,
            screen: ScreenBuffer::new(site.rows_per_page as usize, site.cols_per_page as usize),
            site,
            parser: Parser::new(),
        })
    }

    pub async fn process_incoming(&mut self) -> Result<()> {
        let data = self.connection.recv().await?;
        if data.is_empty() {
            return Ok(());
        }

        let text = decode(&data, self.site.encoding);
        let mut performer = Vt100Performer {
            screen: &mut self.screen,
            cursor_visible: true,
        };

        for b in text.as_bytes() {
            self.parser.advance(&mut performer, *b);
        }
        Ok(())
    }

    pub async fn send_key(&mut self, data: &[u8]) -> Result<()> {
        self.connection.send_raw(data).await
    }
}
