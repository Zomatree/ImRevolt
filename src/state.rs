use std::{any::Any, cell::{Cell, RefCell}, collections::HashMap, sync::{Arc, Mutex}};

use imgui::Ui;

use revolt_database::events::client::EventV1;
use revolt_models::v0::{Channel, Member, Message, Server, User};

use crate::http::{HttpClient, RevoltConfig};

pub struct Context {
    hooks: Vec<Box<dyn Any>>,
    idx: Cell<usize>
}

impl Context {
    fn new() -> Self {
        Self {
            hooks: Vec::new(),
            idx: Cell::new(0)
        }
    }

    pub fn reset(&self) {
        self.idx.set(0);
    }

    pub fn use_hook<F: FnOnce() -> R, R: 'static>(&mut self, func: F) -> &mut R {
        let length = self.hooks.len();

        if self.idx.get() <= length {
            self.hooks.push(Box::new(func()));
        };

        let value = &mut self.hooks[self.idx.get()];
        self.idx.set(length);

        return value.downcast_mut::<R>().unwrap()
    }

    pub fn use_state<F: FnOnce() -> R, R: Clone + 'static>(&mut self, func: F) -> &mut ContextState<R> {
        self.use_hook(|| ContextState::new(func()))
    }
}

#[derive(Clone)]
pub struct ContextState<T> {
    value: Arc<Mutex<T>>
}

impl<T: Clone> ContextState<T> {
    fn new(value: T) -> Self {
        ContextState { value: Arc::new(Mutex::new(value)) }
    }

    pub fn set(&self, value: T) {
        let mut v = self.value.lock().unwrap();
        *v = value;
    }

    pub fn get(&self) -> T {
        self.value.lock().unwrap().clone()
    }
}

enum ConnectionState {
    Disconnected,
    Connected
}

pub struct GlobalState {
    pub config: RevoltConfig,

    pub servers: HashMap<String, Server>,
    pub users: HashMap<String, User>,
    pub members: HashMap<String, HashMap<String, Member>>,
    pub channels: HashMap<String, Channel>,
    pub messages: HashMap<String, Vec<Message>>,

    pub current_message: String,
    pub current_message_hover: Option<String>,
    pub selected_server: Option<String>,
    pub selected_channel: Option<String>,
    pub selected_member: Option<String>,

    pub connection_state: ConnectionState,

    pub contexts: HashMap<String, Context>,
    pub http: HttpClient
}

impl GlobalState {
    pub fn new(base_url: String, config: RevoltConfig, token: String) -> Self {
        Self {
            config,

            servers: HashMap::new(),
            users: HashMap::new(),
            members: HashMap::new(),
            channels: HashMap::new(),
            messages: HashMap::new(),

            current_message: String::new(),
            current_message_hover: None,
            selected_server: None,
            selected_channel: None,
            selected_member: None,

            connection_state: ConnectionState::Disconnected,

            contexts: HashMap::new(),
            http: HttpClient::new(base_url, token)
        }
    }

    pub fn new_context<T: Into<String>>(&mut self, name: T) -> &mut Context {
        let context = self.contexts
            .entry(name.into())
            .or_insert_with(Context::new);

        context.reset();

        context
    }
}

pub fn update_state(event: EventV1, state: &mut GlobalState) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                update_state(e, state)
            }
        },
        EventV1::Authenticated => {
            state.connection_state = ConnectionState::Connected
        },
        EventV1::Logout => {},
        EventV1::Ready { users, servers, channels, members, emojis: _ } => {
            for user in users {
                state.users.insert(user.id.clone(), user);
            };

            for server in servers {
                state.members.insert(server.id.clone(), HashMap::new());
                state.servers.insert(server.id.clone(), server);
            };

            for channel in channels {
                state.messages.insert(channel.id().to_string(), Vec::new());
                state.channels.insert(channel.id().to_string(), channel);
            };

            for member in members {
                state.members.get_mut(&member.id.server)
                    .map(|members| members.insert(member.id.user.clone(), member));
            };
        },
        EventV1::Message(mut message) => {
            if let Some(user) = message.user.take() {
                state.users.insert(user.id.clone(), user);
            };

            if let Some(member) = message.member.take() {
                state.members.get_mut(&member.id.server)
                    .map(|members| members.insert(member.id.user.clone(), member));
            };

            state.messages.get_mut(&message.channel)
                .map(|messages| messages.push(message));
        }
        event => {
            println!("Unhandled Event {:?}", event);
        }
    }
}