use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::{CompositeTemplate, SearchEntry, Button, FlowBox, Box as GtkBox, Picture};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(CompositeTemplate, Default)]
#[template(file = "../../resources/window.ui")]
pub struct MainWindow {
    #[template_child]
    pub search_entry: TemplateChild<SearchEntry>,
    #[template_child]
    pub settings_button: TemplateChild<Button>,
    #[template_child]
    pub app_grid: TemplateChild<FlowBox>,
    #[template_child]
    pub pinned_panel: TemplateChild<GtkBox>,
    #[template_child]
    pub categories_bar: TemplateChild<GtkBox>,
    #[template_child]
    pub background_picture: TemplateChild<Picture>,
    #[template_child]
    pub dim_overlay: TemplateChild<GtkBox>,
    pub current_category: RefCell<Option<Rc<RefCell<String>>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for MainWindow {
    const NAME: &'static str = "MainWindow";
    type Type = super::MainWindow;
    type ParentType = gtk4::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainWindow {}
impl WidgetImpl for MainWindow {}
impl WindowImpl for MainWindow {}