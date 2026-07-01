use gtk4::prelude::*;
use gtk4::{gio, gdk};

/// Прикрепляет контекстное меню (ПКМ) к кнопке
pub fn attach(button: &gtk4::Button, app_id: &str, app_name: &str) {
    let menu_model = gio::Menu::new();
    menu_model.append(Some("Launch"), Some("app_card.launch"));
    menu_model.append(Some("Add to category"), Some("app_card.add_category"));
    menu_model.append(Some("Pin"), Some("app_card.pin"));
    menu_model.append(Some("Hide"), Some("app_card.hide"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(button);

    let action_group = build_actions(app_id, app_name);
    button.insert_action_group("app_card", Some(&action_group));

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(gdk::BUTTON_SECONDARY);

    let popover_clone = popover.clone();
    gesture.connect_pressed(move |gesture, _n_press, x, y| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        popover_clone.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover_clone.popup();
    });

    button.add_controller(gesture);
}

fn build_actions(app_id: &str, app_name: &str) -> gio::SimpleActionGroup {
    let action_group = gio::SimpleActionGroup::new();

    action_group.add_action(&make_action("launch", "Launch", app_id, app_name));
    action_group.add_action(&make_action("add_category", "Add to category", app_id, app_name));
    action_group.add_action(&make_action("pin", "Pin", app_id, app_name));
    action_group.add_action(&make_action("hide", "Hide", app_id, app_name));

    action_group
}

fn make_action(name: &str, log_label: &str, app_id: &str, app_name: &str) -> gio::SimpleAction {
    let action = gio::SimpleAction::new(name, None);
    let id = app_id.to_string();
    let title = app_name.to_string();
    let log_label = log_label.to_string();

    action.connect_activate(move |_, _| {
        println!("[Меню] {} нажат для: {} ({})", log_label, title, id);
    });

    action
}