#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use unicode_width::UnicodeWidthChar;

use crate::terminal::screen::ScreenBuffer;
use crate::terminal::types::COLOR_TABLE;

/// Display settings used by the terminal renderer.
#[derive(Clone, Debug)]
pub struct DisplaySettings {
    pub font_family: String,
    pub font_size: i32,
    pub padding_x: i32,
    pub padding_y: i32,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            font_family: "Monospace".to_string(),
            font_size: 14,
            padding_x: 0,
            padding_y: 0,
        }
    }
}

struct WidgetState {
    screen: ScreenBuffer,
    settings: DisplaySettings,
    show_cursor: bool,
    blink_state: bool,
    /// Cached cell dimensions (pixels).
    char_w: f64,
    char_h: f64,
}

impl WidgetState {
    fn new(screen: ScreenBuffer, settings: DisplaySettings) -> Self {
        Self {
            screen,
            settings,
            show_cursor: true,
            blink_state: false,
            char_w: 8.0,
            char_h: 16.0,
        }
    }

    /// Measure a single character cell using Pango.
    fn measure_cell(
        &mut self,
        cr: &cairo::Context,
    ) {
        let layout = pangocairo::functions::create_layout(cr);
        let mut font_desc = pango::FontDescription::new();
        font_desc.set_family(&self.settings.font_family);
        font_desc.set_size(self.settings.font_size * pango::SCALE);
        layout.set_font_description(Some(&font_desc));
        layout.set_text("M");
        let (pw, ph) = layout.pixel_size();
        self.char_w = pw.max(1) as f64;
        self.char_h = ph.max(1) as f64;
    }
}

/// GTK4 `DrawingArea`-based widget that renders a `ScreenBuffer` using Cairo + Pango.
pub struct TerminalWidget {
    pub drawing_area: gtk4::DrawingArea,
    state: Rc<RefCell<WidgetState>>,
}

impl TerminalWidget {
    pub fn new(screen: ScreenBuffer, settings: DisplaySettings) -> Self {
        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_focusable(true);
        drawing_area.set_can_focus(true);

        let rows = screen.rows;
        let cols = screen.cols;
        let state = Rc::new(RefCell::new(WidgetState::new(screen, settings)));

        // Set a reasonable initial size request.
        drawing_area.set_size_request((cols * 8) as i32, (rows * 16) as i32);

        // ── Draw function ──────────────────────────────────────────────────
        {
            let state = state.clone();
            drawing_area.set_draw_func(move |_area, cr, _width, _height| {
                let mut st = state.borrow_mut();
                st.measure_cell(cr);
                let char_w = st.char_w;
                let char_h = st.char_h;
                let rows = st.screen.rows;
                let cols = st.screen.cols;
                let cursor = st.screen.cursor;
                let show_cursor = st.show_cursor;
                let blink_state = st.blink_state;
                let font_family = st.settings.font_family.clone();
                let font_size = st.settings.font_size;
                let pad_x = st.settings.padding_x as f64;
                let pad_y = st.settings.padding_y as f64;

                // Fill background.
                cr.set_source_rgb(0.0, 0.0, 0.0);
                let _ = cr.paint();

                let layout = pangocairo::functions::create_layout(cr);
                let mut font_desc = pango::FontDescription::new();
                font_desc.set_family(&font_family);
                font_desc.set_size(font_size * pango::SCALE);
                layout.set_font_description(Some(&font_desc));

                for row in 0..rows {
                    for col in 0..cols {
                        let cell = st.screen.cell(row, col).clone();
                        let attr = cell.attr;

                        // Skip invisible blink-off cells.
                        if attr.blink && blink_state && cell.ch != ' ' {
                            // Still draw background, just skip the char below.
                        }

                        // Determine effective fg / bg indices.
                        let (fg_idx, bg_idx) = if attr.inverse {
                            (attr.bg as usize, attr.fg as usize)
                        } else {
                            (attr.fg as usize, attr.bg as usize)
                        };

                        // Bright (bold) bumps low fg colours to high.
                        let fg_idx = if attr.bright && fg_idx < 8 { fg_idx + 8 } else { fg_idx };
                        let fg_idx = fg_idx.min(15);
                        let bg_idx = bg_idx.min(15);

                        let fg_rgb = COLOR_TABLE[fg_idx];
                        let bg_rgb = COLOR_TABLE[bg_idx];

                        let cell_width = if UnicodeWidthChar::width(cell.ch).unwrap_or(1) > 1 {
                            2.0 * char_w
                        } else {
                            char_w
                        };

                        let x = pad_x + col as f64 * char_w;
                        let y = pad_y + row as f64 * char_h;

                        // 1. Background rectangle.
                        cr.set_source_rgb(
                            bg_rgb[0] as f64 / 255.0,
                            bg_rgb[1] as f64 / 255.0,
                            bg_rgb[2] as f64 / 255.0,
                        );
                        cr.rectangle(x, y, cell_width, char_h);
                        let _ = cr.fill();

                        // 2. Foreground character.
                        if !attr.invisible && cell.ch != ' ' && !(attr.blink && blink_state) {
                            cr.set_source_rgb(
                                fg_rgb[0] as f64 / 255.0,
                                fg_rgb[1] as f64 / 255.0,
                                fg_rgb[2] as f64 / 255.0,
                            );
                            cr.move_to(x, y);
                            let s: String = if cell.ch == '\0' { ' ' } else { cell.ch }.to_string();
                            layout.set_text(&s);
                            pangocairo::functions::show_layout(cr, &layout);
                        }

                        // 3. Underline.
                        if attr.underline {
                            cr.set_source_rgb(
                                fg_rgb[0] as f64 / 255.0,
                                fg_rgb[1] as f64 / 255.0,
                                fg_rgb[2] as f64 / 255.0,
                            );
                            cr.move_to(x, y + char_h - 1.0);
                            cr.line_to(x + cell_width, y + char_h - 1.0);
                            let _ = cr.stroke();
                        }

                        // 4. Cursor.
                        if show_cursor && cursor == (row, col) {
                            // Draw an inverted rectangle.
                            cr.set_source_rgb(
                                fg_rgb[0] as f64 / 255.0,
                                fg_rgb[1] as f64 / 255.0,
                                fg_rgb[2] as f64 / 255.0,
                            );
                            cr.rectangle(x, y, char_w, char_h);
                            let _ = cr.fill();

                            if cell.ch != ' ' && cell.ch != '\0' {
                                cr.set_source_rgb(
                                    bg_rgb[0] as f64 / 255.0,
                                    bg_rgb[1] as f64 / 255.0,
                                    bg_rgb[2] as f64 / 255.0,
                                );
                                cr.move_to(x, y);
                                layout.set_text(&cell.ch.to_string());
                                pangocairo::functions::show_layout(cr, &layout);
                            }
                        }
                    }
                }
            });
        }

        // ── Key pressed handler ────────────────────────────────────────────
        // Key events are handled externally via connect_key_pressed on the
        // EventControllerKey attached to the parent window/widget.

        // ── Click / context menu ───────────────────────────────────────────
        let gesture = gtk4::GestureClick::new();
        gesture.set_button(0); // all buttons
        {
            let drawing_area_weak = drawing_area.downgrade();
            gesture.connect_pressed(move |gesture, _n_press, _x, _y| {
                if gesture.current_button() == gdk4::BUTTON_SECONDARY {
                    // Build a simple context menu.
                    if let Some(da) = drawing_area_weak.upgrade() {
                        let menu = gtk4::PopoverMenu::from_model(None::<&gio::MenuModel>);
                        menu.set_parent(&da);
                        menu.popup();
                    }
                }
            });
        }
        drawing_area.add_controller(gesture);

        TerminalWidget { drawing_area, state }
    }

    /// Flip the blink state and queue a redraw.
    pub fn toggle_blink(&self) {
        let mut st = self.state.borrow_mut();
        st.blink_state = !st.blink_state;
        drop(st);
        self.drawing_area.queue_draw();
    }

    pub fn queue_redraw(&self) {
        self.drawing_area.queue_draw();
    }

    /// Replace the internal screen buffer and trigger a redraw.
    pub fn set_screen(&self, screen: ScreenBuffer) {
        self.state.borrow_mut().screen = screen;
        self.drawing_area.queue_draw();
    }

    /// Update the cursor visibility flag.
    pub fn set_show_cursor(&self, visible: bool) {
        self.state.borrow_mut().show_cursor = visible;
    }

    /// Build the VT sequence bytes for a `gdk4::Key` press.
    pub fn key_to_bytes(key: gdk4::Key, modifier: gdk4::ModifierType) -> Option<Vec<u8>> {
        use gdk4::Key;

        // Ctrl + letter → control code 0x01–0x1A
        if modifier.contains(gdk4::ModifierType::CONTROL_MASK) {
            if let Some(c) = key.to_unicode() {
                let cu = c as u32;
                if (b'a' as u32..=b'z' as u32).contains(&cu) {
                    return Some(vec![(cu - b'a' as u32 + 1) as u8]);
                }
                if (b'A' as u32..=b'Z' as u32).contains(&cu) {
                    return Some(vec![(cu - b'A' as u32 + 1) as u8]);
                }
            }
        }

        match key {
            Key::Return | Key::KP_Enter => Some(b"\r".to_vec()),
            Key::BackSpace => Some(vec![0x08]),
            Key::Delete => Some(b"\x1b[3~".to_vec()),
            Key::Escape => Some(vec![0x1b]),
            Key::Tab => Some(vec![0x09]),
            Key::Up => Some(b"\x1b[A".to_vec()),
            Key::Down => Some(b"\x1b[B".to_vec()),
            Key::Right => Some(b"\x1b[C".to_vec()),
            Key::Left => Some(b"\x1b[D".to_vec()),
            Key::Page_Up => Some(b"\x1b[5~".to_vec()),
            Key::Page_Down => Some(b"\x1b[6~".to_vec()),
            Key::Home => Some(b"\x1b[H".to_vec()),
            Key::End => Some(b"\x1b[F".to_vec()),
            Key::F1 => Some(b"\x1bOP".to_vec()),
            Key::F2 => Some(b"\x1bOQ".to_vec()),
            Key::F3 => Some(b"\x1bOR".to_vec()),
            Key::F4 => Some(b"\x1bOS".to_vec()),
            Key::F5 => Some(b"\x1b[15~".to_vec()),
            Key::F6 => Some(b"\x1b[17~".to_vec()),
            Key::F7 => Some(b"\x1b[18~".to_vec()),
            Key::F8 => Some(b"\x1b[19~".to_vec()),
            Key::F9 => Some(b"\x1b[20~".to_vec()),
            Key::F10 => Some(b"\x1b[21~".to_vec()),
            Key::F11 => Some(b"\x1b[23~".to_vec()),
            Key::F12 => Some(b"\x1b[24~".to_vec()),
            other => {
                // Printable characters.
                if let Some(c) = other.to_unicode() {
                    if !c.is_control() {
                        let mut buf = [0u8; 4];
                        let s = c.encode_utf8(&mut buf);
                        return Some(s.as_bytes().to_vec());
                    }
                }
                None
            }
        }
    }
}
