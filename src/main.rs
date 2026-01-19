mod app;
mod window;

use app::MyMarkdownApp;
use gtk::prelude::*;
use std::env;

fn main() {
    // Parse CLI arguments before GTK sees them
    let args: Vec<String> = env::args().collect();
    let file_arg = args.get(1).cloned();

    // Create and run the application
    // Pass empty args to GTK so it doesn't try to handle files
    let app = MyMarkdownApp::new(file_arg);
    app.run_with_args::<String>(&[]);
}
