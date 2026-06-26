#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::config::AppConfig;
use crate::terminal::screen::ScreenBuffer;
use crate::ui::dialogs::preferences::PreferencesDialog;
use crate::ui::dialogs::site_list::SiteListDialog;
use crate::ui::terminal_widget::{DisplaySettings, TerminalWidget};

/// Shared application state held in every closure.
struct AppState {
    config: AppConfig,
    /// One `TerminalWidget` per open tab.
    terminal_widgets: Vec<TerminalWidget>,
}

/// Build and present the main application window.
pub fn build_ui(app: &gtk4::Application) {
    let config = AppConfig::load().unwrap_or_default();

    // ── Top-level window ──────────────────────────────────────────────────
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("PCManX")
        .default_width(config.window.w)
        .default_height(config.window.h)
        .build();

    if config.window.maximized {
        window.maximize();
    }

    // ── Shared state ──────────────────────────────────────────────────────
    let state = Rc::new(RefCell::new(AppState {
        config: config.clone(),
        terminal_widgets: Vec::new(),
    }));

    // ── Root layout ───────────────────────────────────────────────────────
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // ── Menu bar ──────────────────────────────────────────────────────────
    let menu_bar = build_menu_bar();
    root.append(&menu_bar);

    // ── Toolbar ───────────────────────────────────────────────────────────
    let toolbar = build_toolbar();
    root.append(&toolbar);

    // ── Notebook ──────────────────────────────────────────────────────────
    let notebook = gtk4::Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_vexpand(true);
    root.append(&notebook);

    // ── Status bar ────────────────────────────────────────────────────────
    let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    status_bar.set_margin_start(4);
    status_bar.set_margin_end(4);
    status_bar.set_margin_top(2);
    status_bar.set_margin_bottom(2);

    let lbl_status = gtk4::Label::new(Some("Not connected"));
    lbl_status.set_hexpand(true);
    lbl_status.set_halign(gtk4::Align::Start);

    let lbl_encoding = gtk4::Label::new(Some("—"));

    let lbl_clock = gtk4::Label::new(Some(""));
    lbl_clock.set_width_chars(8);

    status_bar.append(&lbl_status);
    status_bar.append(&lbl_encoding);
    status_bar.append(&lbl_clock);
    root.append(&status_bar);

    window.set_child(Some(&root));

    // ── Apply toolbar / status bar / menu bar visibility ──────────────────
    toolbar.set_visible(config.general.show_toolbar);
    status_bar.set_visible(config.general.show_statusbar);
    menu_bar.set_visible(config.general.show_menubar);
    notebook.set_show_tabs(config.general.show_tabbar);

    // ── Toolbar button wiring ─────────────────────────────────────────────
    wire_toolbar_buttons(
        &toolbar,
        &notebook,
        &window,
        &state,
    );

    // ── Keyboard controller ───────────────────────────────────────────────
    {
        let notebook_weak = notebook.downgrade();
        let key_ctrl = gtk4::EventControllerKey::new();
        key_ctrl.connect_key_pressed(move |_, key, _code, modifier| {
            if let Some(nb) = notebook_weak.upgrade() {
                if let Some(page_num) = nb.current_page() {
                    if let Some(child) = nb.nth_page(Some(page_num)) {
                        if let Ok(da) = child.downcast::<gtk4::DrawingArea>() {
                            let _ = da; // TerminalWidget key routing is handled here.
                        }
                    }
                }
            }
            // Convert key to bytes and send via current session.
            let bytes =
                crate::ui::terminal_widget::TerminalWidget::key_to_bytes(key, modifier);
            if let Some(b) = bytes {
                log::debug!("Key bytes: {:?}", b);
                // TODO: route bytes to the active TelnetSession.
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key_ctrl);
    }

    // ── Blink timer (500 ms) ──────────────────────────────────────────────
    {
        let state_weak = Rc::downgrade(&state);
        glib::timeout_add_local(Duration::from_millis(500), move || {
            if let Some(st) = state_weak.upgrade() {
                for tw in &st.borrow().terminal_widgets {
                    tw.toggle_blink();
                }
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    // ── Clock timer (1 s) ─────────────────────────────────────────────────
    {
        let lbl_clock_weak = lbl_clock.downgrade();
        glib::timeout_add_seconds_local(1, move || {
            if let Some(lbl) = lbl_clock_weak.upgrade() {
                let now = glib::DateTime::now_local().unwrap();
                lbl.set_text(&format!("{:02}:{:02}:{:02}",
                    now.hour(), now.minute(), now.second()));
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    // ── Tab switch: update window title and status bar ────────────────────
    {
        let window_weak = window.downgrade();
        let lbl_status_weak = lbl_status.downgrade();
        let lbl_encoding_weak = lbl_encoding.downgrade();
        let state_clone = state.clone();
        notebook.connect_switch_page(move |nb, _page, page_num| {
            if let Some(win) = window_weak.upgrade() {
                if let Some(tab_label) = nb
                    .nth_page(Some(page_num))
                    .and_then(|p| nb.tab_label(&p))
                    .and_then(|tl| tl.downcast::<gtk4::Label>().ok())
                {
                    win.set_title(Some(&format!("PCManX – {}", tab_label.text())));
                }
            }
            if let Some(lbl) = lbl_status_weak.upgrade() {
                lbl.set_text("Connected");
            }
            if let Some(lbl) = lbl_encoding_weak.upgrade() {
                let enc = state_clone
                    .borrow()
                    .config
                    .sites
                    .get(page_num as usize)
                    .map(|s| format!("{:?}", s.encoding))
                    .unwrap_or_else(|| "—".into());
                lbl.set_text(&enc);
            }
        });
    }

    // ── Save window geometry on destroy ───────────────────────────────────
    {
        let state_clone = state.clone();
        window.connect_destroy(move |win| {
            let mut st = state_clone.borrow_mut();
            let (w, h) = (win.width(), win.height());
            st.config.window.w = w;
            st.config.window.h = h;
            st.config.window.maximized = win.is_maximized();
            let _ = st.config.save();
        });
    }

    window.present();
}

// ── Menu bar ──────────────────────────────────────────────────────────────────

fn build_menu_bar() -> gtk4::PopoverMenuBar {
    let menu_model = gio::Menu::new();

    // File
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New Connection"), Some("win.new-connection"));
    file_menu.append(Some("Reconnect"), Some("win.reconnect"));
    file_menu.append(Some("Close Connection"), Some("win.close-connection"));
    file_menu.append(Some("Quit"), Some("app.quit"));
    menu_model.append_submenu(Some("_File"), &file_menu);

    // Edit
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Copy"), Some("win.copy"));
    edit_menu.append(Some("Copy with Color"), Some("win.copy-color"));
    edit_menu.append(Some("Paste"), Some("win.paste"));
    edit_menu.append(Some("Select All"), Some("win.select-all"));
    edit_menu.append(Some("Emojis"), Some("win.emojis"));
    menu_model.append_submenu(Some("_Edit"), &edit_menu);

    // View
    let view_menu = gio::Menu::new();
    view_menu.append(Some("Toggle Toolbar"), Some("win.toggle-toolbar"));
    view_menu.append(Some("Toggle Status Bar"), Some("win.toggle-statusbar"));
    view_menu.append(Some("Toggle Tab Bar"), Some("win.toggle-tabbar"));
    view_menu.append(Some("Toggle Menu Bar"), Some("win.toggle-menubar"));
    view_menu.append(Some("Fullscreen"), Some("win.fullscreen"));
    menu_model.append_submenu(Some("_View"), &view_menu);

    // Sites
    let sites_menu = gio::Menu::new();
    sites_menu.append(Some("Site List"), Some("win.site-list"));
    sites_menu.append(Some("Edit Favorites"), Some("win.edit-favorites"));
    sites_menu.append(Some("Add to Favorites"), Some("win.add-favorite"));
    menu_model.append_submenu(Some("_Sites"), &sites_menu);

    // Tools
    let tools_menu = gio::Menu::new();
    tools_menu.append(Some("Preferences"), Some("win.preferences"));
    menu_model.append_submenu(Some("_Tools"), &tools_menu);

    // Help
    let help_menu = gio::Menu::new();
    help_menu.append(Some("About"), Some("win.about"));
    help_menu.append(Some("Keyboard Shortcuts"), Some("win.shortcuts"));
    menu_model.append_submenu(Some("_Help"), &help_menu);

    gtk4::PopoverMenuBar::from_model(Some(&menu_model))
}

// ── Toolbar ───────────────────────────────────────────────────────────────────

fn build_toolbar() -> gtk4::Box {
    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    toolbar.add_css_class("toolbar");
    toolbar.set_margin_start(4);
    toolbar.set_margin_end(4);
    toolbar.set_margin_top(2);
    toolbar.set_margin_bottom(2);

    let add_btn = |icon: &str, tip: &str| -> gtk4::Button {
        let btn = gtk4::Button::from_icon_name(icon);
        btn.set_tooltip_text(Some(tip));
        btn
    };

    let btn_new = add_btn("network-wired", "New Connection");
    btn_new.set_action_name(Some("win.new-connection"));

    let btn_reconn = add_btn("view-refresh", "Reconnect");
    btn_reconn.set_action_name(Some("win.reconnect"));

    let btn_close = add_btn("window-close", "Close Tab");
    btn_close.set_action_name(Some("win.close-connection"));

    let btn_sites = add_btn("bookmark-new", "Site List");
    btn_sites.set_action_name(Some("win.site-list"));

    let btn_prefs = add_btn("preferences-system", "Preferences");
    btn_prefs.set_action_name(Some("win.preferences"));

    for btn in &[&btn_new, &btn_reconn, &btn_close, &btn_sites, &btn_prefs] {
        toolbar.append(*btn);
    }

    toolbar
}

// ── Toolbar button wiring ─────────────────────────────────────────────────────

fn wire_toolbar_buttons(
    toolbar: &gtk4::Box,
    notebook: &gtk4::Notebook,
    window: &gtk4::ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
) {
    let _ = toolbar;

    // ── win.new-connection ────────────────────────────────────────────────
    {
        let notebook_weak = notebook.downgrade();
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let action_new = gio::SimpleAction::new("new-connection", None);
        action_new.connect_activate(move |_, _| {
            let Some(win) = window_weak.upgrade() else { return };
            let Some(nb) = notebook_weak.upgrade() else { return };

            let config = state_clone.borrow().config.clone();
            let dlg = SiteListDialog::new(win.upcast_ref(), &config);
            dlg.dialog.present();

            let nb_weak2 = nb.downgrade();
            let state2 = state_clone.clone();
            dlg.dialog.connect_destroy(move |_| {
                let Some(nb2) = nb_weak2.upgrade() else { return };
                // Try to use the selected site from the dialog.
                // For now, open a placeholder terminal tab.
                let site = state2.borrow().config.sites.first().cloned()
                    .unwrap_or_default();
                let settings = DisplaySettings {
                    font_family: state2.borrow().config.display.font_family.clone(),
                    font_size: state2.borrow().config.display.font_size,
                    ..Default::default()
                };
                let rows = site.rows_per_page as usize;
                let cols = site.cols_per_page as usize;
                let screen = ScreenBuffer::new(rows, cols);
                let tw = TerminalWidget::new(screen, settings);

                let tab_label = build_tab_label(&site.name, &nb2);
                nb2.append_page(&tw.drawing_area, Some(&tab_label));
                nb2.set_tab_reorderable(&tw.drawing_area, true);
                let page = nb2.n_pages() - 1;
                nb2.set_current_page(Some(page));

                // Kick off async connection in the background.
                let site2 = site.clone();
                glib::spawn_future_local(async move {
                    match crate::ui::terminal_tab::TerminalTab::connect(site2).await {
                        Ok(_tab) => {
                            log::info!("Connected");
                            // TODO: drive process_incoming loop.
                        }
                        Err(e) => log::error!("Connection failed: {e}"),
                    }
                });

                state2.borrow_mut().terminal_widgets.push(tw);
            });
        });
        window.add_action(&action_new);
    }

    // ── win.close-connection ──────────────────────────────────────────────
    {
        let notebook_weak = notebook.downgrade();
        let state_clone = state.clone();
        let action_close = gio::SimpleAction::new("close-connection", None);
        action_close.connect_activate(move |_, _| {
            if let Some(nb) = notebook_weak.upgrade() {
                if let Some(page) = nb.current_page() {
                    nb.remove_page(Some(page));
                    let mut st = state_clone.borrow_mut();
                    if (page as usize) < st.terminal_widgets.len() {
                        st.terminal_widgets.remove(page as usize);
                    }
                }
            }
        });
        window.add_action(&action_close);
    }

    // ── win.reconnect ─────────────────────────────────────────────────────
    {
        let action_reconn = gio::SimpleAction::new("reconnect", None);
        action_reconn.connect_activate(|_, _| {
            log::info!("Reconnect requested");
            // TODO: close and re-open the current session.
        });
        window.add_action(&action_reconn);
    }

    // ── win.site-list ─────────────────────────────────────────────────────
    {
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let action_sites = gio::SimpleAction::new("site-list", None);
        action_sites.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                let config = state_clone.borrow().config.clone();
                let dlg = SiteListDialog::new(win.upcast_ref(), &config);
                dlg.dialog.present();
            }
        });
        window.add_action(&action_sites);
    }

    // ── win.preferences ───────────────────────────────────────────────────
    {
        let window_weak = window.downgrade();
        let state_clone = state.clone();
        let action_prefs = gio::SimpleAction::new("preferences", None);
        action_prefs.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                let config = state_clone.borrow().config.clone();
                let dlg = PreferencesDialog::new(win.upcast_ref(), &config);
                let state2 = state_clone.clone();
                let config_rc = dlg.config.clone();
                dlg.dialog.connect_destroy(move |_| {
                    let new_cfg = config_rc.borrow().clone();
                    state2.borrow_mut().config = new_cfg;
                });
                dlg.dialog.present();
            }
        });
        window.add_action(&action_prefs);
    }

    // ── win.fullscreen ────────────────────────────────────────────────────
    {
        let window_weak = window.downgrade();
        let action_fs = gio::SimpleAction::new("fullscreen", None);
        action_fs.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                if win.is_fullscreen() {
                    win.unfullscreen();
                } else {
                    win.fullscreen();
                }
            }
        });
        window.add_action(&action_fs);
    }

    // ── Stub actions ──────────────────────────────────────────────────────
    for name in &[
        "copy", "copy-color", "paste", "select-all", "emojis",
        "toggle-toolbar", "toggle-statusbar", "toggle-tabbar", "toggle-menubar",
        "edit-favorites", "add-favorite", "about", "shortcuts",
    ] {
        let action = gio::SimpleAction::new(name, None);
        let n = name.to_string();
        action.connect_activate(move |_, _| log::debug!("Action: {n}"));
        window.add_action(&action);
    }

    // ── app.quit ──────────────────────────────────────────────────────────
    {
        let window_weak = window.downgrade();
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(move |_, _| {
            if let Some(win) = window_weak.upgrade() {
                win.close();
            }
        });
        if let Some(app) = window.application() {
            app.add_action(&action_quit);
        }
    }
}

/// Build a tab label widget with the site title.
fn build_tab_label(title: &str, notebook: &gtk4::Notebook) -> gtk4::Box {
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let label = gtk4::Label::new(Some(title));
    let btn_close = gtk4::Button::from_icon_name("window-close");
    btn_close.set_has_frame(false);
    btn_close.set_focusable(false);

    hbox.append(&label);
    hbox.append(&btn_close);

    let notebook_weak = notebook.downgrade();
    let hbox_weak = hbox.downgrade();
    btn_close.connect_clicked(move |_| {
        if let (Some(nb), Some(hb)) = (notebook_weak.upgrade(), hbox_weak.upgrade()) {
            // Find the page whose tab label contains this hbox.
            for i in 0..nb.n_pages() {
                if let Some(child) = nb.nth_page(Some(i)) {
                    if nb.tab_label(&child).as_ref() == Some(hb.upcast_ref()) {
                        nb.remove_page(Some(i));
                        break;
                    }
                }
            }
        }
    });

    hbox
}
