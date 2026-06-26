#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

/// Common BBS emoticons / art symbols for quick insertion.
const EMOTICONS: &[&str] = &[
    ":-)", ":-(", ":-D", ":-P", ":-O", ";-)", ":-|", ":-/",
    "XD", "QQ", "ORZ", "Orz", "orz", "囧", "囧rz", "囧RZ",
    "(>_<)", "(^_^)", "(T_T)", "(=_=)", "(¬_¬)", "(*^▽^*)",
    "(●'◡'●)", "(╯°□°）╯", "~(˘▾˘~)", "ヽ(✿ﾟ▽ﾟ)ノ",
    "( ͡° ͜ʖ ͡°)", "¯\\_(ツ)_/¯", "(ง'̀-'́)ง", "(づ｡◕‿‿◕｡)づ",
];

/// Grid of emoticon buttons.  Clicking one stores the selection and closes.
/// Replaces `emoticondlg.cpp`.
pub struct EmojisDialog {
    pub dialog: gtk4::Window,
    chosen: Rc<RefCell<Option<String>>>,
}

impl EmojisDialog {
    pub fn new(parent: &gtk4::Window) -> Self {
        let chosen: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        let dialog = gtk4::Window::builder()
            .title("Emojis / Emoticons")
            .transient_for(parent)
            .modal(true)
            .default_width(400)
            .default_height(300)
            .build();

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);
        vbox.set_margin_start(8);
        vbox.set_margin_end(8);

        // Flow box for the emoticon grid.
        let flow = gtk4::FlowBox::builder()
            .homogeneous(false)
            .column_spacing(4)
            .row_spacing(4)
            .selection_mode(gtk4::SelectionMode::None)
            .build();

        for emoticon in EMOTICONS {
            let btn = gtk4::Button::with_label(emoticon);
            let emoticon_str = emoticon.to_string();
            let chosen_clone = chosen.clone();
            let dialog_weak = dialog.downgrade();
            btn.connect_clicked(move |_| {
                *chosen_clone.borrow_mut() = Some(emoticon_str.clone());
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            });
            flow.insert(&btn, -1);
        }

        let scroll = gtk4::ScrolledWindow::builder()
            .child(&flow)
            .vexpand(true)
            .build();
        vbox.append(&scroll);

        let btn_cancel = gtk4::Button::with_label("Cancel");
        {
            let dialog_weak = dialog.downgrade();
            btn_cancel.connect_clicked(move |_| {
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
            });
        }
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        btn_row.set_halign(gtk4::Align::End);
        btn_row.append(&btn_cancel);
        vbox.append(&btn_row);

        dialog.set_child(Some(&vbox));

        Self { dialog, chosen }
    }

    /// Present the dialog.  Returns the chosen emoticon *only* if the user
    /// already clicked one before this call returns (which is not typical).
    /// The recommended pattern is to call `dialog.present()` directly and then
    /// connect to `dialog.connect_destroy` to read `self.chosen` after the
    /// window closes.
    pub fn run(&self) -> Option<String> {
        self.dialog.present();
        self.chosen.borrow().clone()
    }
}
