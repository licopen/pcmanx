#![allow(dead_code)]

use log::debug;
use vte::{Params, Parser, Perform};

use super::screen::ScreenBuffer;
use super::types::CharAttr;

pub struct Vt100Performer<'a> {
    pub screen: &'a mut ScreenBuffer,
    pub cursor_visible: bool,
}

impl<'a> Vt100Performer<'a> {
    fn param_or(params: &Params, index: usize, default: usize) -> usize {
        params
            .iter()
            .nth(index)
            .and_then(|p| p.first())
            .map(|v| *v as usize)
            .filter(|v| *v > 0)
            .unwrap_or(default)
    }

    fn sgr(&mut self, params: &Params) {
        if params.is_empty() {
            self.screen.current_attr = CharAttr::default();
            return;
        }

        for param in params.iter() {
            let code = param.first().copied().unwrap_or(0);
            match code {
                0 => self.screen.current_attr = CharAttr::default(),
                1 => self.screen.current_attr.bright = true,
                4 => self.screen.current_attr.underline = true,
                5 => self.screen.current_attr.blink = true,
                7 => self.screen.current_attr.inverse = true,
                8 => self.screen.current_attr.invisible = true,
                22 => self.screen.current_attr.bright = false,
                27 => self.screen.current_attr.inverse = false,
                28 => self.screen.current_attr.invisible = false,
                30..=37 => self.screen.current_attr.fg = (code - 30) as u8,
                40..=47 => self.screen.current_attr.bg = (code - 40) as u8,
                90..=97 => {
                    self.screen.current_attr.fg = (code - 90) as u8;
                    self.screen.current_attr.bright = true;
                }
                100..=107 => {
                    self.screen.current_attr.bg = (code - 100) as u8;
                }
                _ => {}
            }
        }
    }
}

impl<'a> Perform for Vt100Performer<'a> {
    fn print(&mut self, c: char) {
        self.screen.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => self.screen.back(),
            0x09 => self.screen.tab(),
            0x0A => self.screen.line_feed(),
            0x0D => self.screen.carriage_return(),
            0x07 => debug!("terminal bell"),
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        if ignore {
            return;
        }

        match action {
            'A' => {
                let n = Self::param_or(params, 0, 1);
                self.screen.cursor.0 = self.screen.cursor.0.saturating_sub(n);
            }
            'B' => {
                let n = Self::param_or(params, 0, 1);
                self.screen.cursor.0 = (self.screen.cursor.0 + n).min(self.screen.rows - 1);
            }
            'C' => {
                let n = Self::param_or(params, 0, 1);
                self.screen.cursor.1 = (self.screen.cursor.1 + n).min(self.screen.cols - 1);
            }
            'D' => {
                let n = Self::param_or(params, 0, 1);
                self.screen.cursor.1 = self.screen.cursor.1.saturating_sub(n);
            }
            'E' => {
                let n = Self::param_or(params, 0, 1);
                self.screen.cursor.0 = (self.screen.cursor.0 + n).min(self.screen.rows - 1);
                self.screen.cursor.1 = 0;
            }
            'F' => {
                let n = Self::param_or(params, 0, 1);
                self.screen.cursor.0 = self.screen.cursor.0.saturating_sub(n);
                self.screen.cursor.1 = 0;
            }
            'G' => {
                let col = Self::param_or(params, 0, 1).saturating_sub(1);
                self.screen.go_to_xy(col, self.screen.cursor.0);
            }
            'H' | 'f' => {
                let row = Self::param_or(params, 0, 1).saturating_sub(1);
                let col = Self::param_or(params, 1, 1).saturating_sub(1);
                self.screen.go_to_xy(col, row);
            }
            'J' => self.screen.clear_screen(Self::param_or(params, 0, 0) as u8),
            'K' => self.screen.erase_line(Self::param_or(params, 0, 0) as u8),
            'L' => {
                let n = Self::param_or(params, 0, 1);
                let row = self.screen.cursor.0;
                for _ in 0..n {
                    self.screen
                        .cells
                        .insert(row, vec![super::types::TermCell::default(); self.screen.cols]);
                    self.screen.cells.remove(self.screen.scroll_bottom);
                }
            }
            'M' => {
                let n = Self::param_or(params, 0, 1);
                let row = self.screen.cursor.0;
                for _ in 0..n {
                    self.screen.cells.remove(row);
                    self.screen.cells.insert(
                        self.screen.scroll_bottom,
                        vec![super::types::TermCell::default(); self.screen.cols],
                    );
                }
            }
            'P' => self.screen.delete_char(Self::param_or(params, 0, 1)),
            '@' => self.screen.insert_char(Self::param_or(params, 0, 1)),
            'm' => self.sgr(params),
            'r' => {
                let top = Self::param_or(params, 0, 1).saturating_sub(1);
                let bottom = Self::param_or(params, 1, self.screen.rows).saturating_sub(1);
                self.screen.scroll_top = top.min(self.screen.rows - 1);
                self.screen.scroll_bottom = bottom.min(self.screen.rows - 1).max(self.screen.scroll_top);
            }
            's' => self.screen.save_cursor(),
            'u' => self.screen.restore_cursor(),
            'h' | 'l' => {
                let private_mode = intermediates.contains(&b'?');
                if private_mode {
                    let mode = Self::param_or(params, 0, 0);
                    if mode == 25 {
                        self.cursor_visible = action == 'h';
                    }
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'7' => self.screen.save_cursor(),
            b'8' => self.screen.restore_cursor(),
            b'M' => self.screen.scroll_down(1),
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn hook(
        &mut self,
        _params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
}

pub fn process_bytes(screen: &mut ScreenBuffer, data: &[u8]) {
    let mut parser = Parser::new();
    let mut performer = Vt100Performer {
        screen,
        cursor_visible: true,
    };
    for byte in data {
        parser.advance(&mut performer, *byte);
    }
}
