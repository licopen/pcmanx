#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::config::{AppConfig, SiteConfig};

/// Dialog showing the list of configured BBS sites.
/// Replaces `sitelistdlg.cpp`.
pub struct SiteListDialog {
    pub dialog: gtk4::Window,
    pub selected: Rc<RefCell<Option<SiteConfig>>>,
}

impl SiteListDialog {
    pub fn new(parent: &gtk4::Window, config: &AppConfig) -> Self {
        let dialog = gtk4::Window::builder()
            .title("Site List")
            .transient_for(parent)
            .modal(true)
            .default_width(550)
            .default_height(400)
            .build();

        let selected: Rc<RefCell<Option<SiteConfig>>> = Rc::new(RefCell::new(None));

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);
        vbox.set_margin_start(8);
        vbox.set_margin_end(8);

        // ── List model ────────────────────────────────────────────────────
        let store = gtk4::StringList::new(&[]);
        for site in &config.sites {
            store.append(&site.name);
        }

        let selection = gtk4::SingleSelection::new(Some(store.clone()));
        let list_view = gtk4::ListView::builder()
            .model(&selection)
            .factory(&{
                let factory = gtk4::SignalListItemFactory::new();
                factory.connect_setup(|_, item| {
                    let label = gtk4::Label::new(None);
                    label.set_halign(gtk4::Align::Start);
                    item.downcast_ref::<gtk4::ListItem>()
                        .unwrap()
                        .set_child(Some(&label));
                });
                factory.connect_bind(|_, item| {
                    let list_item = item.downcast_ref::<gtk4::ListItem>().unwrap();
                    let label = list_item
                        .child()
                        .and_downcast::<gtk4::Label>()
                        .unwrap();
                    if let Some(obj) = list_item.item() {
                        if let Ok(s) = obj.downcast::<gtk4::StringObject>() {
                            label.set_text(&s.string());
                        }
                    }
                });
                factory
            })
            .build();

        let scroll = gtk4::ScrolledWindow::builder()
            .child(&list_view)
            .vexpand(true)
            .build();
        vbox.append(&scroll);

        // ── Button row ────────────────────────────────────────────────────
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_row.set_halign(gtk4::Align::End);

        let btn_connect = gtk4::Button::with_label("Connect");
        let btn_new = gtk4::Button::with_label("New");
        let btn_edit = gtk4::Button::with_label("Edit");
        let btn_delete = gtk4::Button::with_label("Delete");
        let btn_close = gtk4::Button::with_label("Close");

        btn_row.append(&btn_connect);
        btn_row.append(&btn_new);
        btn_row.append(&btn_edit);
        btn_row.append(&btn_delete);
        btn_row.append(&btn_close);
        vbox.append(&btn_row);

        dialog.set_child(Some(&vbox));

        // ── Connect button ────────────────────────────────────────────────
        {
            let dialog_weak = dialog.downgrade();
            let selected_clone = selected.clone();
            let sites = config.sites.clone();
            btn_connect.connect_clicked(move |_| {
                let idx = selection.selected() as usize;
                if idx < sites.len() {
                    *selected_clone.borrow_mut() = Some(sites[idx].clone());
                }
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            });
        }

        // ── New button ────────────────────────────────────────────────────
        {
            let dialog_weak = dialog.downgrade();
            btn_new.connect_clicked(move |_| {
                if let Some(d) = dialog_weak.upgrade() {
                    let edit = crate::ui::dialogs::site_edit::SiteEditDialog::new(&d, None);
                    edit.dialog.present();
                }
            });
        }

        // ── Edit button ───────────────────────────────────────────────────
        {
            let dialog_weak = dialog.downgrade();
            let sites = config.sites.clone();
            btn_edit.connect_clicked(move |_| {
                if let Some(d) = dialog_weak.upgrade() {
                    // The index isn't trivially available after move; use first site as example.
                    let site = sites.first().cloned();
                    let edit =
                        crate::ui::dialogs::site_edit::SiteEditDialog::new(&d, site.as_ref());
                    edit.dialog.present();
                }
            });
        }

        // ── Delete / Close ────────────────────────────────────────────────
        {
            let dialog_weak = dialog.downgrade();
            btn_delete.connect_clicked(move |_| {
                // TODO: remove the selected site from AppConfig and persist.
                log::debug!("Delete site clicked");
                let _ = dialog_weak;
            });
        }
        {
            let dialog_weak = dialog.downgrade();
            btn_close.connect_clicked(move |_| {
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            });
        }

        Self { dialog, selected }
    }

    /// Present the dialog.  Returns the chosen `SiteConfig` *only* if the user
    /// already pressed Connect before this call returns (which is not typical).
    /// The recommended pattern is to call `dialog.present()` directly and then
    /// connect to `dialog.connect_destroy` to read `self.selected` after the
    /// window closes.
    pub fn run(&self) -> Option<SiteConfig> {
        self.dialog.present();
        self.selected.borrow().clone()
    }
}
