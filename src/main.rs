//! Image processing application for displaying diffs between images.

use std::env::args;

use gio;
use gtk;

use gio::prelude::*;
use gtk::prelude::*;

mod ui;

/// Runs main GTK application loop.
fn main() {
    let application =
        gtk::Application::new("org.bitbucket.glassvis", gio::ApplicationFlags::empty())
            .expect("Failed to load Glassvis!");

    application.connect_startup(move |app| {
        ui::build(app);
    });

    application.connect_activate(|_| {});
    application.run(&args().collect::<Vec<_>>());
}
