use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use pulldown_cmark::{html, Options, Parser};
use sourceview::prelude::*;
use webkit::prelude::*;
use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Write,
    Preview,
    Split,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MyMarkdownWindow {
        pub source_view: RefCell<Option<sourceview::View>>,
        pub web_view: RefCell<Option<webkit::WebView>>,
        pub current_file: RefCell<Option<PathBuf>>,
        pub view_mode: Cell<ViewMode>,
        pub updating: Cell<bool>,
        pub paned: RefCell<Option<gtk::Paned>>,
        pub editor_frame: RefCell<Option<gtk::Frame>>,
        pub preview_frame: RefCell<Option<gtk::Frame>>,
        pub write_btn: RefCell<Option<gtk::ToggleButton>>,
        pub preview_btn: RefCell<Option<gtk::ToggleButton>>,
        pub split_btn: RefCell<Option<gtk::ToggleButton>>,
        pub modified: Cell<bool>,
        pub close_confirmed: Cell<bool>,
        pub initial_dir: RefCell<PathBuf>,
        pub close_after_save: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MyMarkdownWindow {
        const NAME: &'static str = "MyMarkdownWindow";
        type Type = super::MyMarkdownWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for MyMarkdownWindow {}
    impl WidgetImpl for MyMarkdownWindow {}
    impl WindowImpl for MyMarkdownWindow {
        fn close_request(&self) -> glib::Propagation {
            let window = self.obj();

            // If already confirmed or not modified, allow close
            if self.close_confirmed.get() || !self.modified.get() {
                return glib::Propagation::Proceed;
            }

            // Show confirmation dialog
            window.show_close_confirmation();
            glib::Propagation::Stop
        }
    }
    impl ApplicationWindowImpl for MyMarkdownWindow {}
    impl AdwApplicationWindowImpl for MyMarkdownWindow {}
}

glib::wrapper! {
    pub struct MyMarkdownWindow(ObjectSubclass<imp::MyMarkdownWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MyMarkdownWindow {
    pub fn new(app: &crate::app::MyMarkdownApp, file_arg: Option<String>, initial_dir: PathBuf) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("default-width", 1200)
            .property("default-height", 800)
            .build();

        // Store initial directory
        window.imp().initial_dir.replace(initial_dir);

        window.setup_ui();
        window.setup_actions();

        // Set initial mode to Write
        window.set_view_mode(ViewMode::Write);

        // Set write button as active
        if let Some(ref write_btn) = *window.imp().write_btn.borrow() {
            write_btn.set_active(true);
        }
        if let Some(ref split_btn) = *window.imp().split_btn.borrow() {
            split_btn.set_active(false);
        }

        // Handle file argument
        if let Some(filename) = file_arg {
            window.handle_file_arg(&filename);
        } else {
            window.set_title(Some("Untitled - MyMarkdown"));
        }

        window
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Main container
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Header bar
        let header = adw::HeaderBar::new();

        // Title widget
        let title = adw::WindowTitle::new("MyMarkdown", "");
        header.set_title_widget(Some(&title));

        // New button
        let new_btn = gtk::Button::from_icon_name("document-new-symbolic");
        new_btn.set_tooltip_text(Some("New (Ctrl+N)"));
        header.pack_start(&new_btn);

        // Open button
        let open_btn = gtk::Button::from_icon_name("document-open-symbolic");
        open_btn.set_tooltip_text(Some("Open (Ctrl+O)"));
        header.pack_start(&open_btn);

        // Save button
        let save_btn = gtk::Button::from_icon_name("document-save-symbolic");
        save_btn.set_tooltip_text(Some("Save (Ctrl+S)"));
        header.pack_start(&save_btn);

        // View mode toggle buttons [Write] | [Preview]
        let view_toggle_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        view_toggle_box.add_css_class("linked");

        let write_btn = gtk::ToggleButton::with_label("Write");
        write_btn.set_tooltip_text(Some("Write mode (Ctrl+1)"));

        let preview_btn = gtk::ToggleButton::with_label("Preview");
        preview_btn.set_tooltip_text(Some("Preview mode (Ctrl+2)"));
        preview_btn.set_group(Some(&write_btn));

        view_toggle_box.append(&write_btn);
        view_toggle_box.append(&preview_btn);

        header.set_title_widget(Some(&view_toggle_box));

        // Split view toggle button
        let split_btn = gtk::ToggleButton::new();
        split_btn.set_icon_name("view-dual-symbolic");
        split_btn.set_tooltip_text(Some("Toggle Split View (Ctrl+\\)"));
        split_btn.set_active(false);
        header.pack_end(&split_btn);

        // Menu button
        let menu_btn = gtk::MenuButton::new();
        menu_btn.set_icon_name("open-menu-symbolic");
        menu_btn.set_tooltip_text(Some("Menu"));

        let menu = gio::Menu::new();
        menu.append(Some("About"), Some("win.about"));
        menu_btn.set_menu_model(Some(&menu));
        header.pack_end(&menu_btn);

        main_box.append(&header);

        // Create paned view for editor and preview
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_vexpand(true);
        paned.set_hexpand(true);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);
        paned.set_resize_start_child(true);
        paned.set_resize_end_child(true);

        // Editor side
        let editor_frame = self.create_editor();
        paned.set_start_child(Some(&editor_frame));

        // Preview side
        let preview_frame = self.create_preview();
        paned.set_end_child(Some(&preview_frame));

        // Set initial position to 50%
        paned.set_position(600);

        imp.paned.replace(Some(paned.clone()));
        imp.editor_frame.replace(Some(editor_frame));
        imp.preview_frame.replace(Some(preview_frame));
        imp.write_btn.replace(Some(write_btn.clone()));
        imp.preview_btn.replace(Some(preview_btn.clone()));
        imp.split_btn.replace(Some(split_btn.clone()));

        main_box.append(&paned);

        self.set_content(Some(&main_box));

        // Connect signals
        self.connect_signals(&new_btn, &open_btn, &save_btn, &write_btn, &preview_btn, &split_btn);
    }

    fn create_editor(&self) -> gtk::Frame {
        let imp = self.imp();

        let frame = gtk::Frame::new(None);
        frame.add_css_class("view");

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        // Create source view with markdown language
        let buffer = sourceview::Buffer::new(None);

        // Set markdown language for syntax highlighting
        let lang_manager = sourceview::LanguageManager::default();
        if let Some(lang) = lang_manager.language("markdown") {
            buffer.set_language(Some(&lang));
        }

        // Use Adwaita-dark scheme as base (more neutral colors)
        let scheme_manager = sourceview::StyleSchemeManager::default();
        if let Some(scheme) = scheme_manager.scheme("Adwaita-dark")
            .or_else(|| scheme_manager.scheme("classic-dark"))
        {
            buffer.set_style_scheme(Some(&scheme));
        }

        let source_view = sourceview::View::with_buffer(&buffer);
        source_view.set_monospace(true);
        source_view.set_show_line_numbers(true);
        source_view.set_highlight_current_line(true);
        source_view.set_tab_width(4);
        source_view.set_indent_width(4);
        source_view.set_auto_indent(true);
        source_view.set_insert_spaces_instead_of_tabs(true);
        source_view.set_smart_backspace(true);
        source_view.set_wrap_mode(gtk::WrapMode::Word);
        source_view.set_left_margin(12);
        source_view.set_right_margin(12);
        source_view.set_top_margin(12);
        source_view.set_bottom_margin(12);

        // Apply Yaru dark orange theme via CSS
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_string(
            r#"
            /* Toggle buttons (Write/Preview) - orange when active */
            .linked button:checked {
                background: #E95420;
                color: #ffffff;
            }

            /* Split button when active */
            headerbar button:checked {
                background: #E95420;
                color: #ffffff;
            }

            /* Paned separator - orange accent */
            paned > separator {
                background-color: #E95420;
                min-width: 2px;
                min-height: 2px;
            }

            /* Line numbers gutter - dark charcoal with orange text */
            .source-view gutter {
                background-color: #252525;
                padding: 0 8px;
            }

            .source-view .line-numbers {
                background-color: #252525;
                color: #E95420;
                font-weight: 500;
            }

            /* Text selection - orange */
            textview text selection {
                background-color: #E95420;
                color: #ffffff;
            }

            /* Caret color */
            textview text {
                caret-color: #E95420;
            }

            /* Scrollbar hover - orange */
            scrollbar slider:hover {
                background-color: #E95420;
            }
            "#,
        );
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );

        // Setup paste handler for plain text
        self.setup_paste_handler(&source_view);

        // Connect buffer changed signal for live preview and modified tracking
        let window = self.clone();
        buffer.connect_changed(move |_| {
            window.imp().modified.set(true);
            window.update_preview();
        });

        scrolled.set_child(Some(&source_view));
        frame.set_child(Some(&scrolled));

        imp.source_view.replace(Some(source_view));
        frame
    }

    fn create_preview(&self) -> gtk::Frame {
        let imp = self.imp();

        let frame = gtk::Frame::new(None);
        frame.add_css_class("view");

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        // Create webkit view for preview
        let web_view = webkit::WebView::new();
        web_view.set_vexpand(true);
        web_view.set_hexpand(true);

        // Disable editing and JavaScript in preview for security
        if let Some(settings) = webkit::prelude::WebViewExt::settings(&web_view) {
            settings.set_enable_write_console_messages_to_stdout(false);
            settings.set_enable_developer_extras(false);
            settings.set_enable_javascript(false);
            settings.set_enable_javascript_markup(false);
        }

        // Load initial empty content
        self.load_preview_content(&web_view, "");

        scrolled.set_child(Some(&web_view));
        frame.set_child(Some(&scrolled));

        imp.web_view.replace(Some(web_view));
        frame
    }

    fn setup_paste_handler(&self, source_view: &sourceview::View) {
        // Override paste to always use plain text
        let controller = gtk::EventControllerKey::new();
        let view = source_view.clone();

        controller.connect_key_pressed(move |_, key, _, modifier| {
            // Check for Ctrl+V
            if modifier.contains(gdk::ModifierType::CONTROL_MASK)
                && (key == gdk::Key::v || key == gdk::Key::V)
            {
                let clipboard = view.clipboard();
                let view_clone = view.clone();

                // Read plain text from clipboard
                clipboard.read_text_async(None::<&gio::Cancellable>, move |result| {
                    if let Ok(Some(text)) = result {
                        let buffer = view_clone.buffer();
                        buffer.delete_selection(true, true);
                        buffer.insert_at_cursor(&text);
                    }
                });

                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });

        source_view.add_controller(controller);
    }

    fn connect_signals(
        &self,
        new_btn: &gtk::Button,
        open_btn: &gtk::Button,
        save_btn: &gtk::Button,
        write_btn: &gtk::ToggleButton,
        preview_btn: &gtk::ToggleButton,
        split_btn: &gtk::ToggleButton,
    ) {
        // New button
        let window = self.clone();
        new_btn.connect_clicked(move |_| {
            window.new_file();
        });

        // Open button
        let window = self.clone();
        open_btn.connect_clicked(move |_| {
            window.open_file_dialog();
        });

        // Save button
        let window = self.clone();
        save_btn.connect_clicked(move |_| {
            window.save_file();
        });

        // Write button
        let window = self.clone();
        let split_btn_clone = split_btn.clone();
        write_btn.connect_toggled(move |btn| {
            if btn.is_active() && !window.imp().updating.get() {
                window.imp().updating.set(true);
                split_btn_clone.set_active(false);
                window.set_view_mode(ViewMode::Write);
                window.imp().updating.set(false);
            }
        });

        // Preview button
        let window = self.clone();
        let split_btn_clone = split_btn.clone();
        preview_btn.connect_toggled(move |btn| {
            if btn.is_active() && !window.imp().updating.get() {
                window.imp().updating.set(true);
                split_btn_clone.set_active(false);
                window.set_view_mode(ViewMode::Preview);
                window.imp().updating.set(false);
            }
        });

        // Split view toggle
        let window = self.clone();
        split_btn.connect_toggled(move |btn| {
            if !window.imp().updating.get() {
                window.imp().updating.set(true);
                if btn.is_active() {
                    // Deselect write/preview buttons
                    if let Some(ref write_btn) = *window.imp().write_btn.borrow() {
                        write_btn.set_active(false);
                    }
                    if let Some(ref preview_btn) = *window.imp().preview_btn.borrow() {
                        preview_btn.set_active(false);
                    }
                    window.set_view_mode(ViewMode::Split);
                } else {
                    // If turning off split and neither write/preview is selected, select write
                    let write_active = window.imp().write_btn.borrow().as_ref().map(|b| b.is_active()).unwrap_or(false);
                    let preview_active = window.imp().preview_btn.borrow().as_ref().map(|b| b.is_active()).unwrap_or(false);

                    if !write_active && !preview_active {
                        if let Some(ref write_btn) = *window.imp().write_btn.borrow() {
                            write_btn.set_active(true);
                        }
                        window.set_view_mode(ViewMode::Write);
                    }
                }
                window.imp().updating.set(false);
            }
        });
    }

    fn setup_actions(&self) {
        // Keyboard shortcuts
        let app = self.application().unwrap();

        // Ctrl+N - New
        let action = gio::SimpleAction::new("new", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            window.new_file();
        });
        self.add_action(&action);
        app.set_accels_for_action("win.new", &["<Ctrl>n"]);

        // Ctrl+O - Open
        let action = gio::SimpleAction::new("open", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            window.open_file_dialog();
        });
        self.add_action(&action);
        app.set_accels_for_action("win.open", &["<Ctrl>o"]);

        // Ctrl+S - Save
        let action = gio::SimpleAction::new("save", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            window.save_file();
        });
        self.add_action(&action);
        app.set_accels_for_action("win.save", &["<Ctrl>s"]);

        // Ctrl+Shift+S - Save As
        let action = gio::SimpleAction::new("save-as", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            window.save_file_as();
        });
        self.add_action(&action);
        app.set_accels_for_action("win.save-as", &["<Ctrl><Shift>s"]);

        // Ctrl+1 - Write mode
        let action = gio::SimpleAction::new("write-mode", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            if let Some(ref write_btn) = *window.imp().write_btn.borrow() {
                write_btn.set_active(true);
            }
        });
        self.add_action(&action);
        app.set_accels_for_action("win.write-mode", &["<Ctrl>1"]);

        // Ctrl+2 - Preview mode
        let action = gio::SimpleAction::new("preview-mode", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            if let Some(ref preview_btn) = *window.imp().preview_btn.borrow() {
                preview_btn.set_active(true);
            }
        });
        self.add_action(&action);
        app.set_accels_for_action("win.preview-mode", &["<Ctrl>2"]);

        // Ctrl+\ - Toggle Split view
        let action = gio::SimpleAction::new("toggle-split", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            if let Some(ref split_btn) = *window.imp().split_btn.borrow() {
                split_btn.set_active(!split_btn.is_active());
            }
        });
        self.add_action(&action);
        app.set_accels_for_action("win.toggle-split", &["<Ctrl>backslash"]);

        // About action
        let action = gio::SimpleAction::new("about", None);
        let window = self.clone();
        action.connect_activate(move |_, _| {
            window.show_about();
        });
        self.add_action(&action);
    }

    fn set_view_mode(&self, mode: ViewMode) {
        let imp = self.imp();
        imp.view_mode.set(mode);

        if let (Some(editor_frame), Some(preview_frame), Some(paned)) = (
            &*imp.editor_frame.borrow(),
            &*imp.preview_frame.borrow(),
            &*imp.paned.borrow(),
        ) {
            match mode {
                ViewMode::Write => {
                    editor_frame.set_visible(true);
                    preview_frame.set_visible(false);
                }
                ViewMode::Preview => {
                    editor_frame.set_visible(false);
                    preview_frame.set_visible(true);
                    self.update_preview();
                }
                ViewMode::Split => {
                    editor_frame.set_visible(true);
                    preview_frame.set_visible(true);
                    // Set position after window is realized
                    let paned_clone = paned.clone();
                    glib::idle_add_local_once(move || {
                        let width = paned_clone.width();
                        if width > 0 {
                            paned_clone.set_position(width / 2);
                        } else {
                            // Fallback to default
                            paned_clone.set_position(600);
                        }
                    });
                    self.update_preview();
                }
            }
        }
    }

    fn handle_file_arg(&self, filename: &str) {
        let path = if filename.ends_with(".md") {
            PathBuf::from(filename)
        } else {
            PathBuf::from(format!("{}.md", filename))
        };

        if path.exists() {
            // Open existing file
            self.load_file(&path);
        } else {
            // Create new file
            self.imp().current_file.replace(Some(path.clone()));
            self.update_title();
        }
    }

    fn load_file(&self, path: &PathBuf) {
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Some(ref source_view) = *self.imp().source_view.borrow() {
                    let buffer = source_view.buffer();
                    buffer.set_text(&content);
                }
                self.imp().current_file.replace(Some(path.clone()));
                self.update_title();
                // Reset modified after loading
                self.imp().modified.set(false);
            }
            Err(e) => {
                eprintln!("Error loading file: {}", e);
            }
        }
    }

    fn save_file(&self) {
        let imp = self.imp();

        if let Some(ref path) = *imp.current_file.borrow() {
            self.write_file(path);
        } else {
            self.save_file_as();
        }
    }

    fn save_file_as(&self) {
        let dialog = gtk::FileDialog::new();
        dialog.set_title("Save As");

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.md");
        filter.set_name(Some("Markdown files"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        // Set initial folder
        let initial_dir = self.imp().initial_dir.borrow().clone();
        let initial_folder = gio::File::for_path(&initial_dir);
        dialog.set_initial_folder(Some(&initial_folder));

        let window = self.clone();
        dialog.save(Some(&window.clone()), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let path = if path.extension().is_none() {
                        path.with_extension("md")
                    } else {
                        path
                    };
                    window.imp().current_file.replace(Some(path.clone()));
                    window.write_file(&path);
                    window.update_title();

                    // If close was requested after save, close now
                    if window.imp().close_after_save.get() {
                        window.imp().close_after_save.set(false);
                        window.imp().close_confirmed.set(true);
                        window.close();
                    }
                }
            } else {
                // User cancelled save dialog, reset close_after_save flag
                window.imp().close_after_save.set(false);
            }
        });
    }

    fn write_file(&self, path: &PathBuf) {
        if let Some(ref source_view) = *self.imp().source_view.borrow() {
            let buffer = source_view.buffer();
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);

            if let Err(e) = fs::write(path, text.as_str()) {
                eprintln!("Error saving file: {}", e);
            } else {
                // Reset modified flag after successful save
                self.imp().modified.set(false);
            }
        }
    }

    fn new_file(&self) {
        // Check for unsaved changes first
        if self.imp().modified.get() {
            self.show_discard_confirmation(|win| {
                win.do_new_file();
            });
        } else {
            self.do_new_file();
        }
    }

    fn do_new_file(&self) {
        self.imp().current_file.replace(None);
        self.imp().modified.set(false);
        if let Some(ref source_view) = *self.imp().source_view.borrow() {
            source_view.buffer().set_text("");
        }
        self.set_title(Some("Untitled - MyMarkdown"));
        // Reset modified after setting empty text
        self.imp().modified.set(false);
    }

    fn open_file_dialog(&self) {
        // Check for unsaved changes first
        if self.imp().modified.get() {
            self.show_discard_confirmation(|win| {
                win.do_open_file_dialog();
            });
        } else {
            self.do_open_file_dialog();
        }
    }

    fn do_open_file_dialog(&self) {
        let dialog = gtk::FileDialog::new();
        dialog.set_title("Open File");

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");
        filter.set_name(Some("Markdown files"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        let window = self.clone();
        dialog.open(Some(&window.clone()), None::<&gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    window.load_file(&path);
                }
            }
        });
    }

    fn update_title(&self) {
        if let Some(ref path) = *self.imp().current_file.borrow() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            self.set_title(Some(&format!("{} - MyMarkdown", filename)));
        }
    }

    fn update_preview(&self) {
        let imp = self.imp();
        let mode = imp.view_mode.get();

        if mode == ViewMode::Write {
            return;
        }

        if let Some(ref source_view) = *imp.source_view.borrow() {
            let buffer = source_view.buffer();
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);

            if let Some(ref web_view) = *imp.web_view.borrow() {
                self.load_preview_content(web_view, &text);
            }
        }
    }

    fn load_preview_content(&self, web_view: &webkit::WebView, markdown: &str) {
        // Parse markdown to HTML
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Wrap in HTML template with styling (Yaru dark orange theme)
        let full_html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        :root {{
            color-scheme: dark;
        }}

        body {{
            font-family: "Cantarell", "Ubuntu", "Segoe UI", sans-serif;
            font-size: 15px;
            line-height: 1.6;
            padding: 24px;
            max-width: 100%;
            margin: 0;
            background: #1d1d1d;
            color: #f0f0f0;
        }}

        h1, h2, h3, h4, h5, h6 {{
            margin-top: 1.5em;
            margin-bottom: 0.5em;
            font-weight: 600;
            color: #ffffff;
        }}

        h1 {{ font-size: 2em; border-bottom: 2px solid #E95420; padding-bottom: 0.3em; }}
        h2 {{ font-size: 1.5em; border-bottom: 1px solid #3d3d3d; padding-bottom: 0.3em; }}
        h3 {{ font-size: 1.25em; }}
        h4 {{ font-size: 1em; }}

        p {{
            margin: 1em 0;
        }}

        code {{
            font-family: "JetBrainsMono Nerd Font", "JetBrains Mono", "Ubuntu Mono", monospace;
            font-size: 0.9em;
            background: #2d2d2d;
            color: #ff7043;
            padding: 0.2em 0.4em;
            border-radius: 4px;
            word-break: break-word;
        }}

        pre {{
            background: #2d2d2d;
            padding: 16px;
            border-radius: 8px;
            border-left: 3px solid #E95420;
            overflow-x: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}

        pre code {{
            background: none;
            padding: 0;
            color: #f0f0f0;
            white-space: pre-wrap;
            word-wrap: break-word;
            display: block;
        }}

        blockquote {{
            margin: 1em 0;
            padding: 0.5em 1em;
            border-left: 4px solid #E95420;
            background: #252525;
            color: #b0b0b0;
        }}

        a {{
            color: #E95420;
            text-decoration: none;
        }}

        a:hover {{
            color: #ff7043;
            text-decoration: underline;
        }}

        ul, ol {{
            padding-left: 2em;
        }}

        li {{
            margin: 0.5em 0;
        }}

        li::marker {{
            color: #E95420;
        }}

        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 1em 0;
        }}

        table th, table td {{
            border: 1px solid #3d3d3d;
            padding: 8px 12px;
            text-align: left;
        }}

        table th {{
            font-weight: 600;
            background: #E95420;
            color: #ffffff;
        }}

        tr:nth-child(even) {{
            background: #252525;
        }}

        hr {{
            border: none;
            border-top: 2px solid #E95420;
            margin: 2em 0;
        }}

        img {{
            max-width: 100%;
            height: auto;
            border-radius: 8px;
        }}

        ::selection {{
            background: #E95420;
            color: #ffffff;
        }}

        /* Task list */
        ul.task-list {{
            list-style: none;
            padding-left: 1em;
        }}

        input[type="checkbox"] {{
            margin-right: 0.5em;
        }}
    </style>
</head>
<body>
{html_output}
</body>
</html>"#
        );

        web_view.load_html(&full_html, None);
    }

    fn show_about(&self) {
        let about = adw::AboutDialog::builder()
            .application_name("MyMarkdown")
            .application_icon("text-markdown")
            .developer_name("Pan")
            .version("0.1.0")
            .comments("A native GNOME markdown editor with live preview")
            .license_type(gtk::License::MitX11)
            .build();

        about.present(Some(self));
    }

    fn show_discard_confirmation<F>(&self, on_continue: F)
    where
        F: Fn(&Self) + 'static,
    {
        let dialog = adw::AlertDialog::builder()
            .heading("Discard Changes?")
            .body("You have unsaved changes. Do you want to save them first?")
            .close_response("cancel")
            .default_response("save")
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("discard", "Discard");
        dialog.add_response("save", "Save");

        dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

        let window = self.clone();
        dialog.choose(Some(self), None::<&gio::Cancellable>, move |response| {
            match response.as_str() {
                "save" => {
                    // Save first, then continue
                    if window.imp().current_file.borrow().is_some() {
                        window.save_file();
                        on_continue(&window);
                    } else {
                        // Need to save as - this is async, so we can't easily chain
                        // For simplicity, just save and let user try again
                        window.save_file_as();
                    }
                }
                "discard" => {
                    window.imp().modified.set(false);
                    on_continue(&window);
                }
                _ => {
                    // Cancel - do nothing
                }
            }
        });
    }

    fn show_close_confirmation(&self) {
        let dialog = adw::AlertDialog::builder()
            .heading("Save Changes?")
            .body("Do you want to save changes before closing?")
            .close_response("cancel")
            .default_response("save")
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("discard", "Don't Save");
        dialog.add_response("save", "Save");

        dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

        let window = self.clone();
        dialog.choose(Some(self), None::<&gio::Cancellable>, move |response| {
            match response.as_str() {
                "save" => {
                    // Set flag so save_file_as knows to close after save
                    if window.imp().current_file.borrow().is_none() {
                        window.imp().close_after_save.set(true);
                    }
                    window.save_file();
                    // For existing files, close immediately
                    if window.imp().current_file.borrow().is_some() && !window.imp().close_after_save.get() {
                        window.imp().close_confirmed.set(true);
                        window.close();
                    }
                }
                "discard" => {
                    window.imp().close_confirmed.set(true);
                    window.close();
                }
                _ => {
                    // Cancel - do nothing
                }
            }
        });
    }
}
