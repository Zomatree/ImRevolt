use std::env::args;

use http::RevoltConfig;
use imgui::Condition;
use state::GlobalState;

mod setup;
mod components;
mod state;
mod websocket;
mod http;
use revolt_models::v0;
// fn full(ui: &mut RevoltUi, messages: &mut Vec<Message>, current_channel: &mut Option<String>, current_hover: &mut Option<String>, selected_member: &mut Option<String>, current_message: &mut String) {
//     ui.window("Revolt")
//     .size(ui.io().display_size, imgui::Condition::FirstUseEver)
//     .position([0.0, 0.0], imgui::Condition::FirstUseEver)
//     .scroll_bar(false)
//     .scrollable(false)
//     .build(|| {
//         if let Some(table_token) = ui.begin_table_with_flags("content", 3, TableFlags::BORDERS_INNER_V | TableFlags::HIDEABLE | TableFlags::RESIZABLE | TableFlags::REORDERABLE | TableFlags::CONTEXT_MENU_IN_BODY | TableFlags::SCROLL_Y) {
//             ui.table_next_column();

//             components::server_list(ui, current_channel);

//             ui.table_next_column();

//             channel(ui, messages, current_hover, current_channel);

//             ui.table_next_column();

//             members(ui, selected_member);

//             ui.table_next_row();
//             ui.table_next_column();
//             ui.table_next_column();

//             message_box(ui, current_message, messages);

//             table_token.end()
//         }
//     });

// }

static BASE_URL: &'static str = "https://revolt.chat/api";

fn main() {
    let token = args().nth(1).expect("No token given");

    println!("Logging in with token {token}");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _t = rt.enter();

    let api_config = rt.block_on(async {
        reqwest::get(BASE_URL).await.unwrap().json::<RevoltConfig>().await.unwrap()
    });

    setup::init(
        "Revolt",
        {
            let api_config = api_config.clone();
            let token = token.clone();

            || {
                GlobalState::new(BASE_URL.to_string(), api_config, token)
            }
        },
        move |sender | async {
            websocket::run(sender, token, api_config).await;
        },
        state::update_state,
        move |_running, ui, state| {
        ui.window("Channel")
            .menu_bar(true)
            .size([400.0, 600.0], Condition::FirstUseEver)
            .resizable(true)
            .build(|| components::channel(ui, state));

        ui.window("Message Box")
            .size([0.0, 0.0], Condition::FirstUseEver)
            .resizable(true)
            .build(|| components::message_box(ui, state));

        ui.window("Server List")
            .menu_bar(true)
            .size([400.0, 600.0], Condition::FirstUseEver)
            .resizable(true)
            .build(|| components::server_list(ui, state));

        ui.window("Member List")
            .menu_bar(true)
            .size([400.0, 600.0], Condition::FirstUseEver)
            .resizable(true)
            .build(|| components::members(ui, state));
    });
}
