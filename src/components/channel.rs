use imgui::Ui;
use revolt_models::v0::{Channel, Message, Server};

use crate::state::GlobalState;

pub fn message_view(ui: &Ui, state: &mut GlobalState, server: Option<&Server>, message: &Message) {
    ui.group(|| {
        let timestamp_ms = ulid::Ulid::from_string(&message.id).unwrap().timestamp_ms();
        let timestamp = eos::Timestamp::from_milliseconds(timestamp_ms as i64);
        let datetime = eos::DateTime::from_timestamp(timestamp, eos::Utc);
        let created_at = eos::format_dt!("[%H:%M]", datetime).to_string();

        let user = &state.users[&message.author];
        let member = server.map(|s| &state.members[&s.id][&user.id]);

        ui.indent();

        for reply in message.replies.clone().unwrap_or_default() {
            if let Some(message) = state.messages[&message.channel].iter().find(|m| m.id == reply) {
                let user = &state.users[&message.author];
                let member = server.map(|s| &state.members[&s.id][&user.id]);

                let name = message.masquerade
                    .as_ref()
                    .and_then(|masq| masq.name.as_deref())
                    .or(member.and_then(|m| m.nickname.as_deref()))
                    .or(user.display_name.as_deref())
                    .unwrap_or(&user.username);

                ui.text_disabled(name);
                ui.same_line();

                if let Some(content) = message.content.as_ref() {
                    ui.text_disabled(content)
                }

            }
        }

        ui.unindent();

        if state.current_message_hover.as_deref().is_some_and(|v| v == &message.id) {
            ui.text(&created_at);
        } else {
            ui.text_disabled(&created_at);
        }
        ui.same_line();

        let name = message.masquerade
            .as_ref()
            .and_then(|masq| masq.name.as_deref())
            .or(member.and_then(|m| m.nickname.as_deref()))
            .or(user.display_name.as_deref())
            .unwrap_or(&user.username);

        ui.text_colored([0.8, 0.0, 0.0, 1.0], name);
        ui.same_line();

        if let Some(content) = &message.content {
            ui.text_wrapped(content)
        }
    });

    if ui.is_item_hovered() {
        state.current_message_hover = Some(message.id.clone())
    }
}

pub fn channel(ui: &Ui, state: &mut GlobalState) {
    if let Some(selected_channel) = state.selected_channel.as_deref() {
        let channel = &state.channels[selected_channel];

        let server = match channel {
            Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => state.servers.get(server).cloned(),
            _ => None
        };

        let channel_messages = &state.messages[selected_channel].clone();

        let channel_name = match &channel {
            Channel::SavedMessages { .. } => "Saved Messages",
            Channel::DirectMessage { .. } => "DM TODO",
            Channel::Group { name, .. } | Channel::TextChannel { name, .. } | Channel::VoiceChannel { name, .. } => name,
        };

        ui.text_disabled(channel_name);

        ui.child_window("Messages")
        .always_vertical_scrollbar(true)
        .build(|| {
            for message in channel_messages {
                message_view(ui, state, server.as_ref(), &message)
            };
        });
    } else {
        ui.text("No selected channel")
    }
}