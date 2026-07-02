mod imp;
mod app_card;
pub mod categories;
mod pinned;
mod add_to_category;
mod background;
mod dialog_utils;

use gtk4::prelude::*;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::{gio, Application};
use std::cell::RefCell;
use std::rc::Rc;

use crate::apps_db::AppsDb;

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk4::Widget, gtk4::Window,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible,
                    gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native,
                    gtk4::Root, gtk4::ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn populate_apps(&self, db: &Rc<AppsDb>) {
        let current_category: Rc<RefCell<String>> = Rc::new(RefCell::new("All".to_string()));
        *self.imp().current_category.borrow_mut() = Some(current_category.clone());

        background::load_saved(self);
        self.setup_escape_close();

        self.rebuild_apps(db);
        self.rebuild_pinned(db);
        categories::populate(self, db, current_category.clone());
        self.setup_filter(current_category, db.clone());
        self.setup_settings_button();
    }

    pub fn current_category(&self) -> Rc<RefCell<String>> {
        self.imp().current_category.borrow().clone().expect("current_category not initialized")
    }

    fn setup_escape_close(&self) {
        let controller = gtk4::EventControllerKey::new();
        let window_clone = self.clone();

        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                window_clone.close();
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        });

        self.add_controller(controller);

        // дублируем на search_entry на случай, если он перехватывает фокус
        let imp = self.imp();
        let search_controller = gtk4::EventControllerKey::new();
        let window_for_search = self.clone();

        search_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                window_for_search.close();
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
    });

    imp.search_entry.add_controller(search_controller);
    }
    pub fn rebuild_apps(&self, db: &Rc<AppsDb>) {
        let imp = self.imp();

        while let Some(child) = imp.app_grid.first_child() {
            imp.app_grid.remove(&child);
        }

        for app in gio::AppInfo::all() {
            let id = match app.id() {
                Some(id) => id.to_string(),
                None => continue,
            };

            if !app.should_show() {
                continue;
            }

            let hidden = db.is_hidden(&id);
            let card = app_card::build(&app, db, self);
            card.set_widget_name(&format!(
                "{}|{}|{}",
                app.display_name().to_lowercase(),
                hidden,
                id
            ));

            imp.app_grid.append(&card);
        }

        imp.app_grid.invalidate_filter();
    }

    pub fn rebuild_pinned(&self, db: &Rc<AppsDb>) {
        pinned::rebuild(self, db);
    }

    fn setup_filter(&self, current_category: Rc<RefCell<String>>, db: Rc<AppsDb>) {
        let imp = self.imp();

        let search_entry = imp.search_entry.clone();
        let category_for_filter = current_category.clone();
        let db_for_filter = db.clone();

        imp.app_grid.set_filter_func(move |child| {
            let widget_name = child.widget_name();
            let mut parts = widget_name.splitn(3, '|');
            let name = parts.next().unwrap_or_default();
            let hidden = parts.next().unwrap_or("false") == "true";

            let query = search_entry.text().to_lowercase();
            if !query.is_empty() && !name.contains(&query.as_str()) {
                return false;
            }

            let category = category_for_filter.borrow();
            match category.as_str() {
                "All" => !hidden,
                "Hidden" => hidden,
                other => {
                    let app_id = parts.next().unwrap_or_default();
                    !hidden && db_for_filter.app_in_category(app_id, other)
                }
            }
        });

        let app_grid = imp.app_grid.clone();
        imp.search_entry.connect_search_changed(move |_| {
            app_grid.invalidate_filter();
        });
    }

    fn setup_settings_button(&self) {
        let imp = self.imp();
        let button = imp.settings_button.clone();
        let window_clone = self.clone();

        imp.settings_button.connect_clicked(move |_| {
            background::show_menu(&button, &window_clone);
        });
    }
}