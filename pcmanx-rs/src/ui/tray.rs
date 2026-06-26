#![allow(dead_code)]

/// GTK4 `StatusIcon` was removed in GTK4.  This is a stub implementation that
/// logs a warning and returns `None`, with a TODO for a real implementation
/// via `libappindicator3` or a portal.
///
/// TODO: Implement proper system tray support using `libappindicator3` bindings
/// or the XDG Desktop Portal SNI interface when running under a compositor that
/// supports it.
pub struct TrayIcon {
    #[allow(dead_code)]
    _app: gtk4::Application,
}

impl TrayIcon {
    /// Attempt to create a tray icon.  Always returns `None` for now because
    /// GTK4 removed `GtkStatusIcon`.
    pub fn new(app: &gtk4::Application) -> Option<Self> {
        log::warn!(
            "System tray is not available: GTK4 dropped GtkStatusIcon. \
             A proper implementation via libappindicator3 or XDG portal is needed."
        );
        // Return Some to allow callers to hold a handle — but we do nothing.
        Some(Self { _app: app.clone() })
    }

    pub fn set_visible(&self, _visible: bool) {
        // stub
    }

    pub fn set_tooltip(&self, _text: &str) {
        // stub
    }
}
