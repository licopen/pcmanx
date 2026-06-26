#![allow(dead_code)]

use gtk4::prelude::*;

/// Dialog for viewing and saving the current screen content as text.
/// Replaces `downarticledlg.cpp`.
pub struct DownloadArticleDialog {
    pub dialog: gtk4::Window,
}

impl DownloadArticleDialog {
    pub fn new(parent: &gtk4::Window, screen_text: String) -> Self {
        let dialog = gtk4::Window::builder()
            .title("Download Article")
            .transient_for(parent)
            .modal(true)
            .default_width(600)
            .default_height(480)
            .build();

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);
        vbox.set_margin_start(8);
        vbox.set_margin_end(8);

        let text_view = gtk4::TextView::new();
        text_view.set_editable(false);
        text_view.set_monospace(true);
        text_view.buffer().set_text(&screen_text);

        let scroll = gtk4::ScrolledWindow::builder()
            .child(&text_view)
            .vexpand(true)
            .hexpand(true)
            .build();
        vbox.append(&scroll);

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        btn_row.set_halign(gtk4::Align::End);
        let btn_save = gtk4::Button::with_label("Save…");
        let btn_close = gtk4::Button::with_label("Close");
        btn_row.append(&btn_save);
        btn_row.append(&btn_close);
        vbox.append(&btn_row);

        dialog.set_child(Some(&vbox));

        // ── Save button ────────────────────────────────────────────────────
        {
            let dialog_weak = dialog.downgrade();
            let text_view_ref = text_view.clone();
            btn_save.connect_clicked(move |_| {
                if let Some(d) = dialog_weak.upgrade() {
                    let fc = gtk4::FileChooserDialog::new(
                        Some("Save Article"),
                        Some(&d),
                        gtk4::FileChooserAction::Save,
                        &[
                            ("Cancel", gtk4::ResponseType::Cancel),
                            ("Save", gtk4::ResponseType::Accept),
                        ],
                    );
                    fc.set_current_name("article.txt");
                    let tv = text_view_ref.clone();
                    fc.connect_response(move |fc, resp| {
                        if resp == gtk4::ResponseType::Accept {
                            if let Some(path) = fc.file().and_then(|f| f.path()) {
                                let buf = tv.buffer();
                                let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                                let _ = std::fs::write(path, text.as_str());
                            }
                        }
                        fc.close();
                    });
                    fc.present();
                }
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

        Self { dialog }
    }
}
