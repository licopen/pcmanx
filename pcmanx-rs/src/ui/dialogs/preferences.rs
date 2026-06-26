#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::config::AppConfig;

/// Tabbed preferences dialog.
/// Replaces `prefdlg.cpp` + `generalprefpage.cpp` + `keysettingpage.cpp`.
pub struct PreferencesDialog {
    pub dialog: gtk4::Window,
    pub config: Rc<RefCell<AppConfig>>,
}

impl PreferencesDialog {
    pub fn new(parent: &gtk4::Window, config: &AppConfig) -> Self {
        let config_rc = Rc::new(RefCell::new(config.clone()));

        let dialog = gtk4::Window::builder()
            .title("Preferences")
            .transient_for(parent)
            .modal(true)
            .default_width(520)
            .default_height(520)
            .build();

        let notebook = gtk4::Notebook::new();

        // ── General tab ───────────────────────────────────────────────────
        {
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .build();

            let g = &config.general;
            let chk_query_exit = gtk4::CheckButton::with_label("Confirm quit");
            chk_query_exit.set_active(g.query_on_exit);
            let chk_query_close = gtk4::CheckButton::with_label("Confirm close tab");
            chk_query_close.set_active(g.query_on_close_conn);
            let chk_copy_trim = gtk4::CheckButton::with_label("Trim trailing whitespace on copy");
            chk_copy_trim.set_active(g.copy_trim_tail);
            let chk_toolbar = gtk4::CheckButton::with_label("Show toolbar");
            chk_toolbar.set_active(g.show_toolbar);
            let chk_statusbar = gtk4::CheckButton::with_label("Show status bar");
            chk_statusbar.set_active(g.show_statusbar);
            let chk_tabbar = gtk4::CheckButton::with_label("Show tab bar");
            chk_tabbar.set_active(g.show_tabbar);
            let chk_menubar = gtk4::CheckButton::with_label("Show menu bar");
            chk_menubar.set_active(g.show_menubar);
            let chk_notifier = gtk4::CheckButton::with_label("Desktop popup notifier");
            chk_notifier.set_active(g.popup_notifier);
            let chk_mid_click = gtk4::CheckButton::with_label("Middle-click closes tab");
            chk_mid_click.set_active(g.mid_click_as_close);

            let spin_opacity = gtk4::SpinButton::with_range(10.0, 100.0, 1.0);
            spin_opacity.set_value(g.opacity as f64);
            let spin_timeout = gtk4::SpinButton::with_range(1.0, 60.0, 1.0);
            spin_timeout.set_value(g.popup_timeout as f64);

            let entry_browser = gtk4::Entry::builder()
                .text(&config.web_browser)
                .hexpand(true)
                .build();
            let entry_mail = gtk4::Entry::builder()
                .text(&config.mail_client)
                .hexpand(true)
                .build();
            let spin_sock = gtk4::SpinButton::with_range(0.0, 120.0, 1.0);
            spin_sock.set_value(config.socket_timeout as f64);

            let checks: Vec<&gtk4::CheckButton> = vec![
                &chk_query_exit,
                &chk_query_close,
                &chk_copy_trim,
                &chk_toolbar,
                &chk_statusbar,
                &chk_tabbar,
                &chk_menubar,
                &chk_notifier,
                &chk_mid_click,
            ];
            for (i, c) in checks.into_iter().enumerate() {
                grid.attach(c, 0, i as i32, 2, 1);
            }
            let base = 9i32;
            for (off, (lbl, w)) in [
                ("Opacity (%):", spin_opacity.upcast_ref::<gtk4::Widget>()),
                ("Popup timeout (s):", spin_timeout.upcast_ref()),
                ("Web browser:", entry_browser.upcast_ref()),
                ("Mail client:", entry_mail.upcast_ref()),
                ("Socket timeout (s):", spin_sock.upcast_ref()),
            ]
            .iter()
            .enumerate()
            {
                let label = gtk4::Label::builder()
                    .label(*lbl)
                    .halign(gtk4::Align::End)
                    .build();
                grid.attach(&label, 0, base + off as i32, 1, 1);
                grid.attach(*w, 1, base + off as i32, 1, 1);
            }

            // Wire OK (applied at close) — store closures for final collection.
            // For brevity we connect to the dialog destroy signal below.
            let config_rc2 = config_rc.clone();
            dialog.connect_destroy(move |_| {
                let mut c = config_rc2.borrow_mut();
                c.general.query_on_exit = chk_query_exit.is_active();
                c.general.query_on_close_conn = chk_query_close.is_active();
                c.general.copy_trim_tail = chk_copy_trim.is_active();
                c.general.show_toolbar = chk_toolbar.is_active();
                c.general.show_statusbar = chk_statusbar.is_active();
                c.general.show_tabbar = chk_tabbar.is_active();
                c.general.show_menubar = chk_menubar.is_active();
                c.general.popup_notifier = chk_notifier.is_active();
                c.general.mid_click_as_close = chk_mid_click.is_active();
                c.general.opacity = spin_opacity.value() as i32;
                c.general.popup_timeout = spin_timeout.value() as i32;
                c.web_browser = entry_browser.text().to_string();
                c.mail_client = entry_mail.text().to_string();
                c.socket_timeout = spin_sock.value() as u32;
            });

            let scroll = gtk4::ScrolledWindow::builder()
                .child(&grid)
                .vexpand(true)
                .build();
            notebook.append_page(&scroll, Some(&gtk4::Label::new(Some("General"))));
        }

        // ── Terminal tab ──────────────────────────────────────────────────
        {
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .build();

            let t = &config.terminal;
            let spin_rows = gtk4::SpinButton::with_range(1.0, 200.0, 1.0);
            spin_rows.set_value(t.rows_per_page as f64);
            let spin_cols = gtk4::SpinButton::with_range(1.0, 400.0, 1.0);
            spin_cols.set_value(t.cols_per_page as f64);
            let chk_beep = gtk4::CheckButton::with_label("Beep on bell");
            chk_beep.set_active(t.beep_on_bell);

            for (row, (lbl, w)) in [
                ("Rows per page:", spin_rows.upcast_ref::<gtk4::Widget>()),
                ("Cols per page:", spin_cols.upcast_ref()),
            ]
            .iter()
            .enumerate()
            {
                let label = gtk4::Label::builder()
                    .label(*lbl)
                    .halign(gtk4::Align::End)
                    .build();
                grid.attach(&label, 0, row as i32, 1, 1);
                grid.attach(*w, 1, row as i32, 1, 1);
            }
            grid.attach(&chk_beep, 0, 2, 2, 1);

            let config_rc2 = config_rc.clone();
            dialog.connect_destroy(move |_| {
                let mut c = config_rc2.borrow_mut();
                c.terminal.rows_per_page = spin_rows.value() as u32;
                c.terminal.cols_per_page = spin_cols.value() as u32;
                c.terminal.beep_on_bell = chk_beep.is_active();
            });

            notebook.append_page(&grid, Some(&gtk4::Label::new(Some("Terminal"))));
        }

        // ── Display tab ───────────────────────────────────────────────────
        {
            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(6)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .build();

            let d = &config.display;
            let chk_aa = gtk4::CheckButton::with_label("Anti-alias font");
            chk_aa.set_active(d.anti_alias_font);
            let chk_compact = gtk4::CheckButton::with_label("Compact layout");
            chk_compact.set_active(d.compact_layout);

            let spin_pad_x = gtk4::SpinButton::with_range(-10.0, 20.0, 1.0);
            spin_pad_x.set_value(d.char_padding_x as f64);
            let spin_pad_y = gtk4::SpinButton::with_range(-10.0, 20.0, 1.0);
            spin_pad_y.set_value(d.char_padding_y as f64);

            let entry_font = gtk4::Entry::builder().text(&d.font_family).hexpand(true).build();
            let spin_font_sz = gtk4::SpinButton::with_range(6.0, 72.0, 1.0);
            spin_font_sz.set_value(d.font_size as f64);

            let entry_font_en = gtk4::Entry::builder()
                .text(&d.font_family_en)
                .hexpand(true)
                .build();
            let spin_font_sz_en = gtk4::SpinButton::with_range(6.0, 72.0, 1.0);
            spin_font_sz_en.set_value(d.font_size_en as f64);

            grid.attach(&chk_aa, 0, 0, 2, 1);
            grid.attach(&chk_compact, 0, 1, 2, 1);
            for (row, (lbl, w)) in [
                ("Char padding X:", spin_pad_x.upcast_ref::<gtk4::Widget>()),
                ("Char padding Y:", spin_pad_y.upcast_ref()),
                ("CJK font family:", entry_font.upcast_ref()),
                ("CJK font size:", spin_font_sz.upcast_ref()),
                ("ASCII font family:", entry_font_en.upcast_ref()),
                ("ASCII font size:", spin_font_sz_en.upcast_ref()),
            ]
            .iter()
            .enumerate()
            {
                let label = gtk4::Label::builder()
                    .label(*lbl)
                    .halign(gtk4::Align::End)
                    .build();
                grid.attach(&label, 0, 2 + row as i32, 1, 1);
                grid.attach(*w, 1, 2 + row as i32, 1, 1);
            }

            let config_rc2 = config_rc.clone();
            dialog.connect_destroy(move |_| {
                let mut c = config_rc2.borrow_mut();
                c.display.anti_alias_font = chk_aa.is_active();
                c.display.compact_layout = chk_compact.is_active();
                c.display.char_padding_x = spin_pad_x.value() as i32;
                c.display.char_padding_y = spin_pad_y.value() as i32;
                c.display.font_family = entry_font.text().to_string();
                c.display.font_size = spin_font_sz.value() as i32;
                c.display.font_family_en = entry_font_en.text().to_string();
                c.display.font_size_en = spin_font_sz_en.value() as i32;
            });

            notebook.append_page(&grid, Some(&gtk4::Label::new(Some("Display"))));
        }

        // ── Hotkeys tab ───────────────────────────────────────────────────
        {
            let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            vbox.set_margin_top(8);
            vbox.set_margin_bottom(8);
            vbox.set_margin_start(8);
            vbox.set_margin_end(8);

            let h = &config.hotkeys;
            // Each entry maps (display label, mutable field setter).
            // HOTKEY_COUNT must equal the number of rows built here.
            const HOTKEY_COUNT: usize = 19;
            let hotkeys = vec![
                ("Site List", h.key_site_list.clone()),
                ("New Connection 1", h.key_new_conn0.clone()),
                ("New Connection 2", h.key_new_conn1.clone()),
                ("Reconnect 1", h.key_reconn0.clone()),
                ("Reconnect 2", h.key_reconn1.clone()),
                ("Close Tab 1", h.key_close0.clone()),
                ("Close Tab 2", h.key_close1.clone()),
                ("Next Tab", h.key_next_page.clone()),
                ("Previous Tab", h.key_prev_page.clone()),
                ("First Tab", h.key_first_page.clone()),
                ("Last Tab", h.key_last_page.clone()),
                ("Copy 1", h.key_copy0.clone()),
                ("Copy 2", h.key_copy1.clone()),
                ("Paste 1", h.key_paste0.clone()),
                ("Paste 2", h.key_paste1.clone()),
                ("Paste Clipboard", h.key_paste_clipboard.clone()),
                ("Emojis", h.key_emotions.clone()),
                ("Fullscreen", h.key_fullscreen.clone()),
                ("Show Main Window", h.key_show_main_window.clone()),
            ];

            let grid = gtk4::Grid::builder()
                .column_spacing(8)
                .row_spacing(4)
                .build();
            let mut entries: Vec<gtk4::Entry> = Vec::new();
            for (row, (action, key)) in hotkeys.iter().enumerate() {
                let label = gtk4::Label::builder()
                    .label(*action)
                    .halign(gtk4::Align::Start)
                    .hexpand(true)
                    .build();
                let entry = gtk4::Entry::builder().text(key).width_chars(20).build();
                grid.attach(&label, 0, row as i32, 1, 1);
                grid.attach(&entry, 1, row as i32, 1, 1);
                entries.push(entry);
            }

            let scroll = gtk4::ScrolledWindow::builder()
                .child(&grid)
                .vexpand(true)
                .build();
            vbox.append(&scroll);

            let config_rc2 = config_rc.clone();
            dialog.connect_destroy(move |_| {
                let mut c = config_rc2.borrow_mut();
                let vals: Vec<String> = entries.iter().map(|e| e.text().to_string()).collect();
                if vals.len() >= HOTKEY_COUNT {
                    c.hotkeys.key_site_list = vals[0].clone();
                    c.hotkeys.key_new_conn0 = vals[1].clone();
                    c.hotkeys.key_new_conn1 = vals[2].clone();
                    c.hotkeys.key_reconn0 = vals[3].clone();
                    c.hotkeys.key_reconn1 = vals[4].clone();
                    c.hotkeys.key_close0 = vals[5].clone();
                    c.hotkeys.key_close1 = vals[6].clone();
                    c.hotkeys.key_next_page = vals[7].clone();
                    c.hotkeys.key_prev_page = vals[8].clone();
                    c.hotkeys.key_first_page = vals[9].clone();
                    c.hotkeys.key_last_page = vals[10].clone();
                    c.hotkeys.key_copy0 = vals[11].clone();
                    c.hotkeys.key_copy1 = vals[12].clone();
                    c.hotkeys.key_paste0 = vals[13].clone();
                    c.hotkeys.key_paste1 = vals[14].clone();
                    c.hotkeys.key_paste_clipboard = vals[15].clone();
                    c.hotkeys.key_emotions = vals[16].clone();
                    c.hotkeys.key_fullscreen = vals[17].clone();
                    c.hotkeys.key_show_main_window = vals[18].clone();
                }
            });

            notebook.append_page(&vbox, Some(&gtk4::Label::new(Some("Hotkeys"))));
        }

        // ── OK / Cancel ───────────────────────────────────────────────────
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_row.set_halign(gtk4::Align::End);
        btn_row.set_margin_end(8);
        btn_row.set_margin_bottom(8);
        let btn_ok = gtk4::Button::with_label("OK");
        let btn_cancel = gtk4::Button::with_label("Cancel");
        btn_row.append(&btn_ok);
        btn_row.append(&btn_cancel);

        let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        outer.append(&notebook);
        outer.append(&btn_row);
        dialog.set_child(Some(&outer));

        {
            let d = dialog.downgrade();
            btn_ok.connect_clicked(move |_| {
                if let Some(dlg) = d.upgrade() {
                    dlg.close();
                }
            });
        }
        {
            let d = dialog.downgrade();
            btn_cancel.connect_clicked(move |_| {
                if let Some(dlg) = d.upgrade() {
                    dlg.close();
                }
            });
        }

        Self { dialog, config: config_rc }
    }

    /// Return the `AppConfig` as currently set in the dialog widgets.
    pub fn get_config(&self) -> AppConfig {
        self.config.borrow().clone()
    }
}
