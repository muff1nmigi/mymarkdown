use crate::window::MyMarkdownWindow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MyMarkdownApp {
        pub file_arg: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MyMarkdownApp {
        const NAME: &'static str = "MyMarkdownApp";
        type Type = super::MyMarkdownApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MyMarkdownApp {}

    impl ApplicationImpl for MyMarkdownApp {
        fn activate(&self) {
            let app = self.obj();
            let file_arg = self.file_arg.borrow().clone();
            let window = MyMarkdownWindow::new(&app, file_arg);
            window.present();
        }
    }

    impl GtkApplicationImpl for MyMarkdownApp {}
    impl AdwApplicationImpl for MyMarkdownApp {}
}

glib::wrapper! {
    pub struct MyMarkdownApp(ObjectSubclass<imp::MyMarkdownApp>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MyMarkdownApp {
    pub fn new(file_arg: Option<String>) -> Self {
        let app: Self = glib::Object::builder()
            .property("application-id", "org.gnome.MyMarkdown")
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build();

        app.imp().file_arg.replace(file_arg);
        app
    }
}
