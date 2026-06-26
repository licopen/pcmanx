#![allow(dead_code)]

use gtk4::prelude::*;

/// Thin wrapper around `gtk4::Notebook` with tab management helpers.
pub struct TabNotebook {
    pub notebook: gtk4::Notebook,
}

impl Default for TabNotebook {
    fn default() -> Self {
        Self::new()
    }
}

impl TabNotebook {
    pub fn new() -> Self {
        let notebook = gtk4::Notebook::new();
        notebook.set_scrollable(true);
        notebook.set_show_border(false);
        Self { notebook }
    }

    /// Add a tab and return the page number.
    pub fn add_tab(&self, widget: &impl IsA<gtk4::Widget>, title: &str) -> i32 {
        let label = gtk4::Label::new(Some(title));
        self.notebook.append_page(widget, Some(&label)) as i32
    }

    pub fn remove_tab(&self, page_num: i32) {
        self.notebook.remove_page(Some(page_num as u32));
    }

    pub fn current_page(&self) -> Option<i32> {
        self.notebook.current_page().map(|p| p as i32)
    }

    pub fn set_current_page(&self, page_num: i32) {
        self.notebook.set_current_page(Some(page_num as u32));
    }

    pub fn page_count(&self) -> i32 {
        self.notebook.n_pages() as i32
    }

    pub fn set_tab_title(&self, page_num: i32, title: &str) {
        if let Some(child) = self.notebook.nth_page(Some(page_num as u32)) {
            if let Some(tab_label) = self.notebook.tab_label(&child) {
                if let Ok(label) = tab_label.downcast::<gtk4::Label>() {
                    label.set_text(title);
                }
            }
        }
    }
}
