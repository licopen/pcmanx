#![allow(dead_code)]

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::charset::{encode, Encoding};

pub const TC_SE: u8 = 240;
pub const TC_NOP: u8 = 241;
pub const TC_DATA_MARK: u8 = 242;
pub const TC_BREAK: u8 = 243;
pub const TC_INTERRUPT_PROCESS: u8 = 244;
pub const TC_ABORT_OUTPUT: u8 = 245;
pub const TC_ARE_YOU_THERE: u8 = 246;
pub const TC_ERASE_CHARACTER: u8 = 247;
pub const TC_ERASE_LINE: u8 = 248;
pub const TC_GO_AHEAD: u8 = 249;
pub const TC_SB: u8 = 250;
pub const TC_WILL: u8 = 251;
pub const TC_WONT: u8 = 252;
pub const TC_DO: u8 = 253;
pub const TC_DONT: u8 = 254;
pub const TC_IAC: u8 = 255;

pub const TO_ECHO: u8 = 1;
pub const TO_SUPPRESS_GO_AHEAD: u8 = 3;
pub const TO_SUPRESS_GO_AHEAD: u8 = TO_SUPPRESS_GO_AHEAD;
pub const TO_TERMINAL_TYPE: u8 = 24;
pub const TO_IS: u8 = 0;
pub const TO_SEND: u8 = 1;
pub const TO_NAWS: u8 = 31;

pub struct TelnetConnection {
    stream: TcpStream,
    recv_buf: Vec<u8>,
    cols: u16,
    rows: u16,
}

impl TelnetConnection {
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host, port)).await?;
        Ok(Self {
            stream,
            recv_buf: Vec::new(),
            cols: 80,
            rows: 24,
        })
    }

    pub fn set_window_size(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    pub async fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data).await?;
        Ok(())
    }

    async fn send_naws(&mut self) -> Result<()> {
        let will = [TC_IAC, TC_WILL, TO_NAWS];
        let sb = [
            TC_IAC,
            TC_SB,
            TO_NAWS,
            (self.cols >> 8) as u8,
            (self.cols & 0xFF) as u8,
            (self.rows >> 8) as u8,
            (self.rows & 0xFF) as u8,
            TC_IAC,
            TC_SE,
        ];
        self.send_raw(&will).await?;
        self.send_raw(&sb).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        let mut buf = [0_u8; 4096];
        let n = self.stream.read(&mut buf).await?;
        if n == 0 {
            return Ok(Vec::new());
        }

        let mut input = std::mem::take(&mut self.recv_buf);
        input.extend_from_slice(&buf[..n]);

        let mut out = Vec::with_capacity(input.len());
        let mut i = 0usize;

        while i < input.len() {
            let b = input[i];
            if b != TC_IAC {
                out.push(b);
                i += 1;
                continue;
            }

            if i + 1 >= input.len() {
                break;
            }

            let cmd = input[i + 1];
            match cmd {
                TC_IAC => {
                    out.push(TC_IAC);
                    i += 2;
                }
                TC_NOP => {
                    i += 2;
                }
                TC_WILL | TC_WONT | TC_DO | TC_DONT => {
                    if i + 2 >= input.len() {
                        break;
                    }
                    let opt = input[i + 2];
                    if cmd == TC_DO && opt == TO_NAWS {
                        self.send_naws().await?;
                    }
                    // TODO: respond with DONT/WONT for unsupported options.
                    i += 3;
                }
                TC_SB => {
                    i += 2;
                    let mut complete = false;
                    while i + 1 < input.len() {
                        if input[i] == TC_IAC && input[i + 1] == TC_SE {
                            i += 2;
                            complete = true;
                            break;
                        }
                        i += 1;
                    }
                    if !complete {
                        i = i.saturating_sub(2);
                        break;
                    }
                }
                _ => {
                    i += 2;
                }
            }
        }

        if i < input.len() {
            self.recv_buf.extend_from_slice(&input[i..]);
        }

        Ok(out)
    }

    pub async fn send_string(&mut self, s: &str, enc: &Encoding) -> Result<()> {
        let data = encode(s, *enc);
        self.send_raw(&data).await
    }

    pub fn is_connected(&self) -> bool {
        self.stream.peer_addr().is_ok()
    }
}
