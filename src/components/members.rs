
use imgui::{StyleColor, Ui};
use revolt_models::v0::{Presence, UserStatus};

use crate::state::GlobalState;



pub fn members(ui: &Ui, state: &mut GlobalState) {
    ui.child_window("Members")
    .horizontal_scrollbar(false)
    .build(|| {
        if let Some(selected_server) = &state.selected_server {
            let members: &std::collections::HashMap<String, revolt_models::v0::Member> = &state.members[selected_server];

            ui.text_disabled(format!("{} Members", members.len()));

            for member in members.values() {
                let user = &state.users[&member.id.user];

                if ui.selectable(member.nickname.as_ref().or(user.display_name.as_ref()).unwrap_or(&user.username)) {
                    state.selected_member = Some(member.id.user.clone());

                    ui.open_popup("member")
                }

                if let Some(_t) = ui.begin_popup_context_item() {
                    if ui.menu_item("Ban") {}
                    if ui.menu_item("Kick") {}
                    if ui.menu_item("Timeout") {}
                    if ui.menu_item("Manage Roles") {}

                    _t.end()
                }
            };

            ui.modal_popup_config("member")
            .save_settings(true)
            .always_auto_resize(false)
            .menu_bar(true)
            .collapsible(true)
            .build(|| {
                if let Some(member_id) = &state.selected_member {
                    let user = &state.users[member_id];
                    let member = &state.members[selected_server][member_id];

                    ui.menu_bar(|| {
                        ui.menu("User", || {
                            if ui.menu_item("Send Friend Request") {}
                            if ui.menu_item("Block") {}
                            if ui.menu_item("Send Message") {}
                            if ui.menu_item("Copy ID") {}
                        });

                        ui.menu("Moderation", || {
                            if ui.menu_item("Ban") {}
                            if ui.menu_item("Kick") {}
                            if ui.menu_item("Timeout") {}
                            if ui.menu_item("Manage Roles") {}
                        })
                    });

                    if let Some(_tabbar_token) = ui.tab_bar("member_tabbar") {
                        if let Some(_tabitem_token) = ui.tab_item("User") {
                            if let Some(display_name) = &user.display_name {
                                ui.text(display_name);
                            }

                            ui.text_disabled(&user.username);
                            ui.same_line_with_spacing(0.0, 0.0);
                            ui.text_disabled(format!("#{}", user.discriminator));

                            if let Some(status) = &user.status {
                                let presence = match &status.presence {
                                    Some(Presence::Online) => "Online",
                                    Some(Presence::Idle) => "Idle",
                                    Some(Presence::Busy) => "Busy",
                                    Some(Presence::Focus) => "Focus",
                                    Some(Presence::Invisible) => "Offline",
                                    None => "Offline"
                                };

                                ui.text_disabled(presence);

                                if let Some(text) = &status.text {
                                    ui.same_line();
                                    ui.text_disabled(text)
                                }
                            }

                            ui.new_line();

                        };

                        if let Some(_tabitem_token) = ui.tab_item("Roles") {
                            ui.child_window("Roles")
                                .build(|| {
                                    for (color, role) in &[([1.0, 0.0, 0.0, 1.0], "Admin"), ([0.0, 1.0, 0.0, 1.0], "Moderator"), ([0.0, 0.0, 1.0, 1.0], "User")] {
                                        let color_token = ui.push_style_color(StyleColor::Text, *color);

                                        if ui.button(role) {}

                                        color_token.end();
                                    };

                                });

                        };
                    }

                    if ui.button("Close") {
                        ui.close_current_popup()
                    }
                } else {
                    ui.text("Invalid State")
                }
            });
        } else {
            ui.text("No selected server")
        }
    });

}