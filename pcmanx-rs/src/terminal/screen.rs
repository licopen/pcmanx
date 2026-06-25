#![allow(dead_code)]

use unicode_width::UnicodeWidthChar;

use super::types::{CharAttr, TermCell};

#[derive(Clone, Debug)]
pub struct ScreenBuffer {
    pub rows: usize,
    pub cols: usize,
    pub(crate) cells: Vec<Vec<TermCell>>,
    pub cursor: (usize, usize),
    pub scroll_top: usize,
    pub scroll_bottom: usize,
    pub saved_cursor: Option<(usize, usize)>,
    pub current_attr: CharAttr,
}

impl ScreenBuffer {
    pub fn new(rows: usize, cols: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            rows,
            cols,
            cells: vec![vec![TermCell::default(); cols]; rows],
            cursor: (0, 0),
            scroll_top: 0,
            scroll_bottom: rows - 1,
            saved_cursor: None,
            current_attr: CharAttr::default(),
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> &TermCell {
        &self.cells[row][col]
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut TermCell {
        &mut self.cells[row][col]
    }

    pub fn put_char(&mut self, ch: char) {
        let (row, col) = self.cursor;
        if row >= self.rows || col >= self.cols {
            return;
        }

        let mut attr = self.current_attr;
        attr.need_update = true;
        self.cells[row][col] = TermCell { ch, attr };

        let width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);
        let next_col = col + width;
        if next_col >= self.cols {
            self.carriage_return();
            self.line_feed();
        } else {
            self.cursor.1 = next_col;
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor.1 = 0;
    }

    pub fn line_feed(&mut self) {
        if self.cursor.0 >= self.scroll_bottom {
            self.scroll_up(1);
        } else {
            self.cursor.0 = (self.cursor.0 + 1).min(self.rows - 1);
        }
    }

    pub fn tab(&mut self) {
        let next_tab = ((self.cursor.1 / 8) + 1) * 8;
        let target = next_tab.min(self.cols.saturating_sub(1));
        while self.cursor.1 < target {
            self.put_char(' ');
        }
    }

    pub fn back(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
    }

    pub fn go_to_xy(&mut self, col: usize, row: usize) {
        self.cursor = (
            row.min(self.rows.saturating_sub(1)),
            col.min(self.cols.saturating_sub(1)),
        );
    }

    pub fn scroll_up(&mut self, n: usize) {
        let n = n.max(1);
        for _ in 0..n {
            if self.scroll_top <= self.scroll_bottom && self.scroll_bottom < self.rows {
                self.cells.remove(self.scroll_top);
                self.cells
                    .insert(self.scroll_bottom, vec![TermCell::default(); self.cols]);
            }
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let n = n.max(1);
        for _ in 0..n {
            if self.scroll_top <= self.scroll_bottom && self.scroll_bottom < self.rows {
                self.cells.remove(self.scroll_bottom);
                self.cells
                    .insert(self.scroll_top, vec![TermCell::default(); self.cols]);
            }
        }
    }

    pub fn erase_line(&mut self, mode: u8) {
        let row = self.cursor.0;
        let col = self.cursor.1;
        match mode {
            0 => {
                for c in col..self.cols {
                    self.cells[row][c] = TermCell::default();
                }
            }
            1 => {
                for c in 0..=col.min(self.cols - 1) {
                    self.cells[row][c] = TermCell::default();
                }
            }
            2 => {
                for c in 0..self.cols {
                    self.cells[row][c] = TermCell::default();
                }
            }
            _ => {}
        }
    }

    pub fn clear_screen(&mut self, mode: u8) {
        match mode {
            0 => {
                self.erase_line(0);
                for r in self.cursor.0 + 1..self.rows {
                    for c in 0..self.cols {
                        self.cells[r][c] = TermCell::default();
                    }
                }
            }
            1 => {
                self.erase_line(1);
                for r in 0..self.cursor.0 {
                    for c in 0..self.cols {
                        self.cells[r][c] = TermCell::default();
                    }
                }
            }
            2 => {
                for r in 0..self.rows {
                    for c in 0..self.cols {
                        self.cells[r][c] = TermCell::default();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn insert_char(&mut self, n: usize) {
        let n = n.max(1).min(self.cols);
        let (row, col) = self.cursor;
        for _ in 0..n {
            self.cells[row].insert(col, TermCell::default());
            self.cells[row].pop();
        }
    }

    pub fn delete_char(&mut self, n: usize) {
        let n = n.max(1).min(self.cols);
        let (row, col) = self.cursor;
        for _ in 0..n {
            if col < self.cols {
                self.cells[row].remove(col);
                self.cells[row].push(TermCell::default());
            }
        }
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor);
    }

    pub fn restore_cursor(&mut self) {
        if let Some((r, c)) = self.saved_cursor {
            self.go_to_xy(c, r);
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        let mut new_cells = vec![vec![TermCell::default(); cols]; rows];
        for (r, row) in new_cells.iter_mut().enumerate().take(self.rows.min(rows)) {
            for (c, cell) in row.iter_mut().enumerate().take(self.cols.min(cols)) {
                *cell = self.cells[r][c].clone();
            }
        }
        self.rows = rows;
        self.cols = cols;
        self.cells = new_cells;
        self.cursor.0 = self.cursor.0.min(rows - 1);
        self.cursor.1 = self.cursor.1.min(cols - 1);
        self.scroll_top = 0;
        self.scroll_bottom = rows - 1;
    }

    pub fn get_line_text(&self, row: usize) -> String {
        self.cells[row].iter().map(|cell| cell.ch).collect()
    }

    pub fn get_all_text(&self) -> String {
        (0..self.rows)
            .map(|r| self.get_line_text(r))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
