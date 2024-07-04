use imgui::Ui;
use revolt_models::v0::DataMessageSend;

use crate::state::GlobalState;


pub fn message_box(ui: &Ui, state: &mut GlobalState) {
    let selected_channel = state.selected_channel.clone();
    let http = state.http.clone();

    let context = state.new_context("MessageBox");
    let current_message = context.use_hook(String::new);

    let should_send = ui.input_text("##textinput", current_message)
        .hint("Message Channel")
        .enter_returns_true(true)
        .build();

    ui.same_line();

    if (ui.button("Send") || should_send) && !current_message.is_empty() {
        if let Some(channel_id) = selected_channel {
            tokio::spawn({
                let current_message = current_message.clone();

                async move {
                    http.send_message(&channel_id, &DataMessageSend {
                        content: Some(current_message),
                        nonce: None,
                        attachments: None,
                        replies: None,
                        embeds: None,
                        masquerade: None,
                        interactions: None,
                        flags: None,
                    }).await.unwrap();
                }
            });
            // Send Message Request Here
            current_message.clear();
        }
    }
}