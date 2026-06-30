mod imp;

use gtk4::prelude::*;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::{gio, Application};

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

    pub fn populate_apps(&self, db: &crate::apps_db::AppsDb) {
        let imp = self.imp();
        let apps = gio::AppInfo::all();

        for app in apps {
            let id = match app.id() {
                Some(id) => id.to_string(),
                None => continue,
            };

            // скрываем твои hidden
            if db.is_hidden(&id) {
                continue;
            }

            // скрываем системные/служебные .desktop файлы
            // should_show() учитывает NoDisplay=true, Hidden=true,
            // OnlyShowIn/NotShowIn относительно текущего DE
            if !app.should_show() {
                continue;
            }

            let name = app.display_name();

            let image = gtk4::Image::new();
            image.set_pixel_size(48);

            if let Some(icon) = app.icon() {
                image.set_from_gicon(&icon);
            } else {
                image.set_icon_name(Some("application-x-executable"));
            }

            let label = gtk4::Label::new(Some(&name));
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_max_width_chars(12);
            label.set_lines(2);
            label.set_wrap(true);
            label.set_justify(gtk4::Justification::Center);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            content.append(&image);
            content.append(&label);

            let button = gtk4::Button::new();
            button.set_child(Some(&content));
            button.add_css_class("flat");
            button.set_width_request(80);
            button.set_height_request(80);

            // запуск приложения по клику
            let app_clone = app.clone();
            button.connect_clicked(move |_| {
                let context = gio::AppLaunchContext::NONE;
                if let Err(e) = app_clone.launch(&[], context) {
                    eprintln!("Не удалось запустить приложение: {}", e);
                }
            });

            // FlowBoxChild создаём вручную, чтобы прицепить к нему имя
            // приложения как "search key" через widget-name для фильтра
            let flow_child = gtk4::FlowBoxChild::new();
            flow_child.set_child(Some(&button));
            flow_child.set_widget_name(&name.to_lowercase());

            imp.app_grid.append(&flow_child);
        }

        self.setup_search();
    }

    fn setup_search(&self) {
        let imp = self.imp();

        // фильтр-функция: показываем только те карточки,
        // чьё имя (widget_name) содержит текст поиска
        let search_entry = imp.search_entry.clone();
        let app_grid = imp.app_grid.clone();

        imp.app_grid.set_filter_func(move |child| {
            let query = search_entry.text().to_lowercase();
            if query.is_empty() {
                return true;
            }
            child.widget_name().to_lowercase().contains(&query)
        });

        // при изменении текста — пересчитываем фильтр
        let app_grid_for_signal = app_grid.clone();
        imp.search_entry.connect_search_changed(move |_| {
            app_grid_for_signal.invalidate_filter();
        });
    }
}