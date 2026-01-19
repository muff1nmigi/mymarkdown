mod app;
mod window;

use app::MyMarkdownApp;
use gtk::prelude::*;
use std::env;

fn main() {
    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    let file_arg = args.get(1).cloned();

    // Create and run the application
    let app = MyMarkdownApp::new(file_arg);
    app.run();
}
