use gtk4::prelude::*;
use gtk4::Application;

mod charset;
mod config;
mod connection;
mod notifier;
mod script;
mod terminal;
mod ui;

const APP_ID: &str = "org.pcmanx.PCManX";

fn main() {
    env_logger::init();
    let app = Application::builder()
        .application_id(APP_ID)
        .build();
    app.connect_activate(ui::main_window::build_ui);
    std::process::exit(app.run().into());
}
