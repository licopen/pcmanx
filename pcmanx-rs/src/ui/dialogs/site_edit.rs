#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::charset::Encoding;
use crate::config::{
    AutoLogin, CrLfMode, ProxyConfig, ProxyType, SiteConfig,
};

/// Dialog for creating or editing a `SiteConfig`.
/// Replaces `sitedlg.cpp` + `editfavdlg.cpp`.
pub struct SiteEditDialog {
    pub dialog: gtk4::Window,
    site: Rc<RefCell<SiteConfig>>,
}

impl SiteEditDialog {
    pub fn new(parent: &gtk4::Window, existing: Option<&SiteConfig>) -> Self {
        let base = existing.cloned().unwrap_or_default();
        let site = Rc::new(RefCell::new(base.clone()));

        let dialog = gtk4::Window::builder()
            .title(if existing.is_some() { "Edit Site" } else { "New Site" })
            .transient_for(parent)
            .modal(true)
            .default_width(480)
            .default_height(500)
            .build();

        let notebook = gtk4::Notebook::new();

        // ── Basic tab ─────────────────────────────────────────────────────
        let basic_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let entry_name = gtk4::Entry::builder().text(&base.name).hexpand(true).build();
        let entry_url = gtk4::Entry::builder().text(&base.url).hexpand(true).build();

        let combo_enc = gtk4::ComboBoxText::new();
        for label in &["Big5", "UAO 2.41", "UAO 2.50", "UTF-8"] {
            combo_enc.append_text(label);
        }
        let enc_idx = match base.encoding {
            Encoding::Big5 => 0,
            Encoding::Uao241 => 1,
            Encoding::Uao250 => 2,
            Encoding::Utf8 => 3,
        };
        combo_enc.set_active(Some(enc_idx));

        let entry_term = gtk4::Entry::builder()
            .text(&base.term_type)
            .hexpand(true)
            .build();

        for (row, (lbl, w)) in [
            ("Name:", entry_name.upcast_ref::<gtk4::Widget>()),
            ("URL (host:port):", entry_url.upcast_ref()),
            ("Encoding:", combo_enc.upcast_ref()),
            ("Terminal type:", entry_term.upcast_ref()),
        ]
        .iter()
        .enumerate()
        {
            let label = gtk4::Label::builder()
                .label(*lbl)
                .halign(gtk4::Align::End)
                .build();
            basic_grid.attach(&label, 0, row as i32, 1, 1);
            basic_grid.attach(*w, 1, row as i32, 1, 1);
        }
        notebook.append_page(&basic_grid, Some(&gtk4::Label::new(Some("Basic"))));

        // ── Login tab ─────────────────────────────────────────────────────
        let login_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let al = &base.auto_login;
        let entry_pre_login = gtk4::Entry::builder().text(&al.pre_login).hexpand(true).build();
        let entry_user = gtk4::Entry::builder().text(&al.login).hexpand(true).build();
        let entry_pass = gtk4::Entry::builder()
            .text(&al.passwd)
            .visibility(false)
            .hexpand(true)
            .build();
        let entry_post = gtk4::Entry::builder().text(&al.post_login).hexpand(true).build();
        let spin_reconnect = gtk4::SpinButton::with_range(0.0, 3600.0, 1.0);
        spin_reconnect.set_value(base.auto_reconnect as f64);
        let spin_idle = gtk4::SpinButton::with_range(0.0, 3600.0, 1.0);
        spin_idle.set_value(base.anti_idle as f64);

        for (row, (lbl, w)) in [
            ("Pre-login:", entry_pre_login.upcast_ref::<gtk4::Widget>()),
            ("Username:", entry_user.upcast_ref()),
            ("Password:", entry_pass.upcast_ref()),
            ("Post-login:", entry_post.upcast_ref()),
            ("Auto-reconnect (s):", spin_reconnect.upcast_ref()),
            ("Anti-idle (s):", spin_idle.upcast_ref()),
        ]
        .iter()
        .enumerate()
        {
            let label = gtk4::Label::builder()
                .label(*lbl)
                .halign(gtk4::Align::End)
                .build();
            login_grid.attach(&label, 0, row as i32, 1, 1);
            login_grid.attach(*w, 1, row as i32, 1, 1);
        }
        notebook.append_page(&login_grid, Some(&gtk4::Label::new(Some("Login"))));

        // ── Display tab ───────────────────────────────────────────────────
        let disp_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let spin_rows = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
        spin_rows.set_value(base.rows_per_page as f64);
        let spin_cols = gtk4::SpinButton::with_range(1.0, 300.0, 1.0);
        spin_cols.set_value(base.cols_per_page as f64);

        let combo_crlf = gtk4::ComboBoxText::new();
        for lbl in &["CR", "LF", "CR+LF"] {
            combo_crlf.append_text(lbl);
        }
        let crlf_idx = match base.crlf {
            CrLfMode::Cr => 0,
            CrLfMode::Lf => 1,
            CrLfMode::CrLf => 2,
        };
        combo_crlf.set_active(Some(crlf_idx));

        for (row, (lbl, w)) in [
            ("Rows:", spin_rows.upcast_ref::<gtk4::Widget>()),
            ("Cols:", spin_cols.upcast_ref()),
            ("CRLF mode:", combo_crlf.upcast_ref()),
        ]
        .iter()
        .enumerate()
        {
            let label = gtk4::Label::builder()
                .label(*lbl)
                .halign(gtk4::Align::End)
                .build();
            disp_grid.attach(&label, 0, row as i32, 1, 1);
            disp_grid.attach(*w, 1, row as i32, 1, 1);
        }
        notebook.append_page(&disp_grid, Some(&gtk4::Label::new(Some("Display"))));

        // ── Proxy tab ─────────────────────────────────────────────────────
        let proxy_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let combo_proxy = gtk4::ComboBoxText::new();
        for lbl in &["None", "SOCKS4", "SOCKS5", "HTTP"] {
            combo_proxy.append_text(lbl);
        }
        let proxy = base.proxy.clone().unwrap_or_default();
        let proxy_type_idx = match proxy.proxy_type {
            ProxyType::None => 0,
            ProxyType::Socks4 => 1,
            ProxyType::Socks5 => 2,
            ProxyType::Http => 3,
        };
        combo_proxy.set_active(Some(proxy_type_idx));

        let entry_proxy_addr = gtk4::Entry::builder().text(&proxy.addr).hexpand(true).build();
        let entry_proxy_port = gtk4::Entry::builder().text(&proxy.port).hexpand(true).build();
        let entry_proxy_user = gtk4::Entry::builder().text(&proxy.user).hexpand(true).build();
        let entry_proxy_pass = gtk4::Entry::builder()
            .text(&proxy.pass)
            .visibility(false)
            .hexpand(true)
            .build();

        for (row, (lbl, w)) in [
            ("Proxy type:", combo_proxy.upcast_ref::<gtk4::Widget>()),
            ("Address:", entry_proxy_addr.upcast_ref()),
            ("Port:", entry_proxy_port.upcast_ref()),
            ("User:", entry_proxy_user.upcast_ref()),
            ("Password:", entry_proxy_pass.upcast_ref()),
        ]
        .iter()
        .enumerate()
        {
            let label = gtk4::Label::builder()
                .label(*lbl)
                .halign(gtk4::Align::End)
                .build();
            proxy_grid.attach(&label, 0, row as i32, 1, 1);
            proxy_grid.attach(*w, 1, row as i32, 1, 1);
        }
        notebook.append_page(&proxy_grid, Some(&gtk4::Label::new(Some("Proxy"))));

        // ── OK / Cancel ───────────────────────────────────────────────────
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_row.set_halign(gtk4::Align::End);
        btn_row.set_margin_top(8);
        btn_row.set_margin_bottom(8);
        btn_row.set_margin_end(8);
        let btn_ok = gtk4::Button::with_label("OK");
        let btn_cancel = gtk4::Button::with_label("Cancel");
        btn_row.append(&btn_ok);
        btn_row.append(&btn_cancel);

        let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        outer.append(&notebook);
        outer.append(&btn_row);
        dialog.set_child(Some(&outer));

        // ── OK handler: collect widget values into SiteConfig ─────────────
        {
            let site_clone = site.clone();
            let dialog_weak = dialog.downgrade();
            let entry_name = entry_name.clone();
            let entry_url = entry_url.clone();
            let combo_enc = combo_enc.clone();
            let entry_term = entry_term.clone();
            let entry_pre_login = entry_pre_login.clone();
            let entry_user = entry_user.clone();
            let entry_pass = entry_pass.clone();
            let entry_post = entry_post.clone();
            let spin_reconnect = spin_reconnect.clone();
            let spin_idle = spin_idle.clone();
            let spin_rows = spin_rows.clone();
            let spin_cols = spin_cols.clone();
            let combo_crlf = combo_crlf.clone();
            let combo_proxy = combo_proxy.clone();
            let entry_proxy_addr = entry_proxy_addr.clone();
            let entry_proxy_port = entry_proxy_port.clone();
            let entry_proxy_user = entry_proxy_user.clone();
            let entry_proxy_pass = entry_proxy_pass.clone();

            btn_ok.connect_clicked(move |_| {
                let encoding = match combo_enc.active() {
                    Some(1) => Encoding::Uao241,
                    Some(2) => Encoding::Uao250,
                    Some(3) => Encoding::Utf8,
                    _ => Encoding::Big5,
                };
                let crlf = match combo_crlf.active() {
                    Some(1) => CrLfMode::Lf,
                    Some(2) => CrLfMode::CrLf,
                    _ => CrLfMode::Cr,
                };
                let proxy_type = match combo_proxy.active() {
                    Some(1) => ProxyType::Socks4,
                    Some(2) => ProxyType::Socks5,
                    Some(3) => ProxyType::Http,
                    _ => ProxyType::None,
                };
                let proxy = ProxyConfig {
                    proxy_type,
                    addr: entry_proxy_addr.text().to_string(),
                    port: entry_proxy_port.text().to_string(),
                    user: entry_proxy_user.text().to_string(),
                    pass: entry_proxy_pass.text().to_string(),
                };

                let mut s = site_clone.borrow_mut();
                s.name = entry_name.text().to_string();
                s.url = entry_url.text().to_string();
                s.encoding = encoding;
                s.term_type = entry_term.text().to_string();
                s.auto_login = AutoLogin {
                    pre_login: entry_pre_login.text().to_string(),
                    login: entry_user.text().to_string(),
                    passwd: entry_pass.text().to_string(),
                    post_login: entry_post.text().to_string(),
                    ..Default::default()
                };
                s.auto_reconnect = spin_reconnect.value() as u32;
                s.anti_idle = spin_idle.value() as u32;
                s.rows_per_page = spin_rows.value() as u32;
                s.cols_per_page = spin_cols.value() as u32;
                s.crlf = crlf;
                s.proxy = if proxy.proxy_type == ProxyType::None {
                    None
                } else {
                    Some(proxy)
                };

                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            });
        }

        {
            let dialog_weak = dialog.downgrade();
            btn_cancel.connect_clicked(move |_| {
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            });
        }

        Self { dialog, site }
    }

    /// Return the current `SiteConfig` as configured in the dialog.
    pub fn get_site(&self) -> SiteConfig {
        self.site.borrow().clone()
    }
}
