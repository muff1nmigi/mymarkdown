mod app;
mod window;

use app::MyMarkdownApp;
use gtk::prelude::*;
use std::env;
use std::path::PathBuf;

fn main() {
    // Parse CLI arguments before GTK sees them
    let args: Vec<String> = env::args().collect();
    let file_arg = args.get(1).cloned();

    // Get current working directory for save dialog
    let initial_dir = env::current_dir().unwrap_or_else(|_| {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    });

    // Create and run the application
    // Pass empty args to GTK so it doesn't try to handle files
    let app = MyMarkdownApp::new(file_arg, initial_dir);
    app.run_with_args::<String>(&[]);
}
