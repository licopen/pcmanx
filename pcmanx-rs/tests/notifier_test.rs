/// Notifier::new creates without panic.
#[test]
fn notifier_new_does_not_panic() {
    let n = pcmanx_rs::notifier::Notifier::new(5, None);
    // We can't actually send a notification in CI (no D-Bus session),
    // but constructing the struct must never panic.
    let _ = n;
}

#[test]
fn notifier_with_icon_does_not_panic() {
    let n = pcmanx_rs::notifier::Notifier::new(3, Some("/usr/share/icons/hicolor/48x48/apps/pcmanx.png".to_string()));
    let _ = n;
}
