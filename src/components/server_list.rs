use imgui::Ui;
use revolt_models::v0::Channel;

use crate::state::GlobalState;


pub fn channel_button(ui: &Ui, state: &mut GlobalState, server_id: &str, channel_id: &str) {
    if let Some(channel) = state.channels.get(channel_id) {
        let name = match channel {
            Channel::SavedMessages { .. } => "Saved Messages",
            Channel::DirectMessage { .. } => "DM TODO",
            Channel::Group { name, .. } | Channel::TextChannel { name, .. } | Channel::VoiceChannel { name, .. } => name,
        };

        if ui.selectable(name) {
            state.selected_channel = Some(channel_id.to_string());
            state.selected_server = Some(server_id.to_string());
        };
    }
}

pub fn server_list(ui: &Ui, state: &mut GlobalState) {
    ui.menu_bar(|| {
        ui.menu("Servers", || {

        });

        ui.menu("Direct Messages", || {

        });
    });

    ui.child_window("Servers")
    .build(|| {
        ui.tree_node_config("Direct Messages")
        .default_open(false)
        .build(|| {

        });

        ui.tree_node_config("Servers")
        .default_open(true)
        .build(|| {
            for server in state.servers.clone().values() {
                if let Some(token) = ui.tree_node(&server.name) {
                    let categories = server.categories.clone().unwrap_or_default();
                    let channels = &server.channels;

                    for channel_id in channels {
                        if !categories.iter().any(|c| c.channels.contains(channel_id)) {

                            channel_button(ui, state, &server.id, channel_id)
                        }
                    }

                    for category in categories {
                        if let Some(cat_token) = ui.tree_node(category.title) {
                            for channel_id in &category.channels {
                                channel_button(ui, state, &server.id, channel_id)
                            }

                            cat_token.end()
                        }
                    }

                    token.end()
                }

            }
        });
    });
}