use core::panic;
use std::{collections::HashMap, default};

/*
* The state of the view can be
*  - password entry for the vault
*  - displaying the schema
*  - viewing an existing entry
*  - creating a new entry
*
*/
use iced::{
    alignment,
    futures::{channel::mpsc, SinkExt},
    theme,
    widget::{button, column, container, pick_list, row, scrollable, text, text_input},
    Application, Command, Element, Length, Settings, Subscription, Theme,
};
use pants_gen::password::Password;
use pants_store::{
    gui::connection,
    message::Message,
    output::Output,
    reads::Reads,
    schema::Schema,
    store::{Store, StoreChoice},
    vault::interface::VaultInterface,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
enum GUIMessage {
    EntryMessage(EntryMessage, String),
    ExitEntry,
    ShowPassword,
    HidePassword,
    CopyPassword,
    ClearPassword,
    PasswordChanged(String),
    PasswordSubmit,
    NewEntry,
    ChangeName(String),
    SelectStyle(StoreChoice),
    UpdateField(String, String),
    GeneratePassword,
    NewCreate,
    Event(connection::Event),
    Send(Message),
}

struct VaultState {
    schema: Schema,
    entries: Vec<Entry>,
    password_state: PasswordState,
    entry_state: Option<EntryState>,
    new_state: Option<NewEntryState>,
    temp_message: TempMessage,
    state: ConnectionState,
}

impl Default for VaultState {
    fn default() -> Self {
        Self {
            schema: Schema::default(),
            entries: vec![],
            password_state: PasswordState::default(),
            entry_state: None,
            new_state: None,
            temp_message: TempMessage::default(),
            state: ConnectionState::Disconnected,
        }
    }
}

enum ConnectionState {
    Disconnected,
    Connected(connection::Connection),
}

impl VaultState {
    fn new(schema: Schema) -> Self {
        let mut entries = vec![];
        for (key, value) in schema.data.iter() {
            entries.push(Entry::new(key.to_string(), value.to_string()));
        }
        Self {
            schema,
            entries,
            ..Default::default()
        }
    }
    fn needs_password(&self) -> bool {
        self.temp_message.needs_password()
    }
    fn update(&mut self, schema: Schema) {
        let mut entries = vec![];
        for (key, value) in schema.data.iter() {
            entries.push(Entry::new(key.to_string(), value.to_string()));
        }

        self.schema = schema;
        self.entries = entries;
    }
    fn update_entry(&mut self, data: Reads<Store>) {
        if let Some(ref mut entry) = &mut self.entry_state {
            for (key, value) in data.data {
                if key == entry.key {
                    entry.update(value);
                }
            }
        }
    }
    fn send_message(&mut self, messages: Vec<Message>) {
        for message in messages {
            match self.state {
                ConnectionState::Disconnected => {}
                ConnectionState::Connected(ref mut connection) => {
                    connection.send(message);
                }
            }
        }
    }
    fn view(&self) -> Element<GUIMessage> {
        if self.password_state.is_active() {
            let header = text("Vault password:").size(20);
            let password_input = text_input(
                "vault password",
                &self
                    .password_state
                    .password
                    .clone()
                    .unwrap_or("".to_string()),
            )
            .on_input(GUIMessage::PasswordChanged)
            .on_submit(GUIMessage::PasswordSubmit)
            .width(Length::Fill)
            .secure(true);
            let info = self.temp_message.view();
            container(column![header, password_input, info]).into()
        } else if let Some(new_entry) = &self.new_state {
            new_entry.view()
        } else if let Some(entry) = &self.entry_state {
            entry.view()
            // let header = text(entry.key).size(50);
            // container(header).into()
        } else {
            let header = text("Entries")
                .size(20)
                .horizontal_alignment(alignment::Horizontal::Center);
            let content = scrollable(column(self.entries.iter().map(|e| {
                e.view()
                    .map(move |message| GUIMessage::EntryMessage(message, e.key.clone()))
            })));
            let new_button = button(
                text("+")
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .width(Length::Fill),
            )
            .width(Length::Fill)
            .on_press(GUIMessage::NewEntry);
            let info = self.temp_message.view();
            container(column![header, content, new_button, info]).into()
        }
    }
}

fn clear_password_command() -> Command<GUIMessage> {
    Command::perform(async { () }, |_| GUIMessage::ClearPassword)
}

#[derive(Debug, Clone, Default)]
enum TempMessage {
    #[default]
    Empty,
    Delete(String),
    Get(String),
    New(String, StoreChoice, HashMap<String, String>),
}

impl TempMessage {
    fn needs_password(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Delete(_) => true,
            Self::Get(_) => true,
            Self::New(_, _, _) => true,
        }
    }

    fn complete(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::New(name, _, fields) => {
                let mut filled = true;
                for (_, value) in fields.iter() {
                    if value.is_empty() {
                        filled = false;
                        break;
                    }
                }
                !name.is_empty() && filled
            }
            Self::Get(name) => !name.is_empty(),
            Self::Delete(name) => !name.is_empty(),
        }
    }

    fn final_command(&self) -> Command<GUIMessage> {
        match self {
            Self::Delete(_) => clear_password_command(),
            Self::New(_, _, _) => clear_password_command(),
            _ => Command::none(),
        }
    }

    fn with_password(&self, password: String) -> Message {
        match self {
            Self::Delete(key) => Message::Delete(password, key.to_string()),
            Self::Get(key) => Message::Get(password, key.to_string()),
            Self::New(key, choice, value) => {
                Message::Update(password, key.clone(), choice.convert(value).unwrap())
            }
            Self::Empty => Message::Schema,
        }
    }

    fn view(&self) -> Element<GUIMessage> {
        match self {
            TempMessage::Delete(key) => {
                let info = text(format!("Working on deleting {}", key));
                container(info).into()
            }
            TempMessage::Get(key) => {
                let info = text(format!("Working on getting {}", key));
                container(info).into()
            }
            TempMessage::New(key, _, _) => {
                let info = text(format!("Working on a new entry {}", key));
                container(info).into()
            }
            Self::Empty => {
                let info = text(format!("Working on nothing"));
                container(info).into()
            }
        }
    }
}

#[derive(Debug, Clone)]
struct NewEntryState {
    name: String,
    choice: StoreChoice,
    value: HashMap<String, String>,
}

impl Default for NewEntryState {
    fn default() -> Self {
        NewEntryState {
            name: String::new(),
            choice: StoreChoice::default(),
            value: StoreChoice::default().convert_default().as_hash(),
        }
    }
}

impl NewEntryState {
    fn view(&self) -> Element<GUIMessage> {
        let name_prefix = text("Name:");
        let name_input = text_input("Name", &self.name).on_input(GUIMessage::ChangeName);
        let style_choice = pick_list(
            StoreChoice::all(),
            Some(self.choice),
            GUIMessage::SelectStyle,
        );
        let data_input = match &self.choice {
            StoreChoice::Password => {
                let prefix = text("Password:");
                let password_input = text_input("Password", &self.value.get("password").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("password".to_string(), v));
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);

                container(row![prefix, password_input, password_generate])
            }
            StoreChoice::UsernamePassword => {
                let username_prefix = text("Username:");
                let password_prefix = text("Password:");
                let username_input = text_input("Username", &self.value.get("username").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("username".to_string(), v));
                let password_input = text_input("Password", &self.value.get("password").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("password".to_string(), v));

                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(column![
                    row![username_prefix, username_input],
                    row![password_prefix, password_input, password_generate]
                ])
            }
        };
        let create_button = button("Create").on_press(GUIMessage::NewCreate);
        let cancel_button = button("Cancel");
        container(column![
            row![name_prefix, name_input],
            style_choice,
            data_input,
            row![create_button, cancel_button]
        ])
        .into()
    }
}

#[derive(Debug, Clone)]
struct EntryState {
    key: String,
    value: Store,
    hidden: bool,
}

impl EntryState {
    fn view(&self) -> Element<GUIMessage> {
        let header = text(self.key.clone()).size(20);
        let content = match &self.value {
            Store::Password(p) => {
                let prefix = text("Password:");
                let password_input = text_input("Password", &p)
                    .width(Length::Fill)
                    .secure(self.hidden);
                let show_button = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                let copy_button = button("Copy").on_press(GUIMessage::CopyPassword);
                container(row![prefix, password_input, copy_button, show_button])
            }
            Store::UsernamePassword(username, password) => {
                let username_prefix = text("Username:");
                let password_prefix = text("Password:");
                let username_input = text_input("Username", &username).width(Length::Fill);
                let password_input = text_input("Password", &password)
                    .width(Length::Fill)
                    .secure(self.hidden);
                let show_button = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                let copy_button = button("Copy").on_press(GUIMessage::CopyPassword);
                container(column![
                    row![username_prefix, username_input],
                    row![password_prefix, password_input, copy_button, show_button]
                ])
            }
        };
        let done_button = button("Done").on_press(GUIMessage::ExitEntry);

        container(column![header, content, done_button]).into()
    }

    fn update(&mut self, value: Store) {
        self.value = value;
    }

    fn get_password(&self) -> Option<String> {
        match &self.value {
            Store::Password(p) => Some(p.to_string()),
            Store::UsernamePassword(_, p) => Some(p.to_string()),
        }
    }

    fn from_entry(key: String, style: String) -> Self {
        let value = match style.as_str() {
            "password" => Store::Password(String::new()),
            "username-password" => Store::UsernamePassword(String::new(), String::new()),
            _ => todo!(),
        };
        EntryState {
            key,
            value,
            hidden: true,
        }
    }
}

#[derive(Debug, Clone)]
struct PasswordState {
    prompt: bool,
    password: Option<String>,
}

impl Default for PasswordState {
    fn default() -> Self {
        Self {
            prompt: false,
            password: None,
        }
    }
}

impl PasswordState {
    fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }
    fn activate(&mut self) {
        self.prompt = true;
    }
    fn deactivate(&mut self) {
        self.prompt = false;
    }
    fn is_active(&self) -> bool {
        self.prompt
    }
    fn clear(&mut self) {
        self.password = None;
    }
}

#[derive(Debug, Clone)]
struct Entry {
    id: Uuid,
    key: String,
    style: String,
}

#[derive(Debug, Clone)]
enum EntryMessage {
    Delete,
    View,
}

impl Entry {
    fn new(key: String, style: String) -> Self {
        Entry {
            id: Uuid::new_v4(),
            key,
            style,
        }
    }

    fn view(&self) -> Element<EntryMessage> {
        let value = text(self.key.clone()).width(Length::Fill);
        let view_button = button("View").on_press(EntryMessage::View);
        let delete_button = button("Delete")
            .on_press(EntryMessage::Delete)
            .style(theme::Button::Destructive);
        let content = row![view_button, value, delete_button];
        container(content)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }
}

#[derive(Debug, Clone)]
enum GUIError {
    LoadSchema,
    Receive,
}

impl Application for VaultState {
    type Flags = ();
    type Theme = Theme;
    type Message = GUIMessage;
    type Executor = iced::executor::Default;

    fn title(&self) -> String {
        "Pants".to_string()
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            GUIMessage::Event(event) => match event {
                connection::Event::Connected(connection) => {
                    self.state = ConnectionState::Connected(connection);
                    self.send_message(vec![Message::Schema]);
                }
                connection::Event::Disconnected => {
                    self.state = ConnectionState::Disconnected;
                }
                connection::Event::ReceiveSchema(schema) => {
                    println!("Received schema: {}", schema);
                    self.update(schema);
                }
                connection::Event::ReceiveRead(value) => {
                    println!("Received read: {}", value);
                    self.update_entry(value);
                }
                _ => {}
            },
            GUIMessage::Send(message) => self.send_message(vec![message]),
            GUIMessage::EntryMessage(EntryMessage::Delete, key) => {
                self.temp_message = TempMessage::Delete(key);
                if self.needs_password() {
                    self.password_state.activate();
                }
            }
            GUIMessage::EntryMessage(EntryMessage::View, key) => {
                self.temp_message = TempMessage::Get(key.clone());
                self.entry_state = Some(EntryState::from_entry(
                    key.to_string(),
                    self.schema.get(&key).unwrap().to_string(),
                ));
                if self.needs_password() {
                    self.password_state.activate();
                }
            }
            GUIMessage::PasswordChanged(p) => {
                self.password_state.password = Some(p);
            }
            GUIMessage::ClearPassword => {
                self.password_state.clear();
            }
            GUIMessage::ChangeName(n) => {
                if let Some(ref mut new_state) = &mut self.new_state {
                    new_state.name = n.clone();
                }
                if let TempMessage::New(ref mut key, _, _) = &mut self.temp_message {
                    *key = n;
                }
            }
            GUIMessage::SelectStyle(choice) => {
                if let Some(ref mut new_state) = &mut self.new_state {
                    new_state.choice = choice;
                    new_state.value = choice.convert_default().as_hash();
                }
                if let TempMessage::New(_, ref mut style, ref mut value) = &mut self.temp_message {
                    *style = choice;
                    *value = choice.convert_default().as_hash();
                }
            }
            GUIMessage::UpdateField(k, v) => {
                if let Some(ref mut new_state) = &mut self.new_state {
                    new_state.value.insert(k.clone(), v.clone());
                }
                if let TempMessage::New(_, _, ref mut value) = &mut self.temp_message {
                    value.insert(k, v);
                }
            }
            GUIMessage::GeneratePassword => {
                let password = Password::default().generate().unwrap();
                if let Some(ref mut new_state) = &mut self.new_state {
                    new_state
                        .value
                        .insert("password".to_string(), password.clone());
                }
                if let TempMessage::New(_, _, ref mut value) = &mut self.temp_message {
                    value.insert("password".to_string(), password);
                }
            }
            GUIMessage::NewCreate => {
                if let Some(new_state) = &self.new_state {
                    if !self.schema.data.contains_key(&new_state.name)
                        && self.temp_message.complete()
                    {
                        self.password_state.activate();
                        self.new_state = None;
                    }
                }
            }
            GUIMessage::NewEntry => {
                self.temp_message = TempMessage::New(
                    String::new(),
                    StoreChoice::default(),
                    StoreChoice::default().convert_default().as_hash(),
                );
                self.new_state = Some(NewEntryState::default());
            }
            GUIMessage::PasswordSubmit => {
                let mut deactivate = false;
                let mut messages = vec![];
                let mut command = Command::none();
                if let Some(password) = &self.password_state.password {
                    deactivate = true;
                    let message = self.temp_message.with_password(password.to_string());
                    messages.push(message);
                    messages.push(Message::Schema);
                    command = self.temp_message.final_command();
                }
                if deactivate {
                    self.password_state.deactivate();
                    self.temp_message = TempMessage::default();
                }

                self.send_message(messages);

                return command;
            }
            GUIMessage::ExitEntry => {
                self.entry_state = None;
                return clear_password_command();
            }
            GUIMessage::ShowPassword => {
                if let Some(ref mut entry) = &mut self.entry_state {
                    entry.hidden = false;
                }
            }
            GUIMessage::HidePassword => {
                if let Some(ref mut entry) = &mut self.entry_state {
                    entry.hidden = true;
                }
            }
            GUIMessage::CopyPassword => {
                if let Some(ref entry) = &self.entry_state {
                    if let Some(p) = entry.get_password() {
                        return iced::clipboard::write(p);
                    }
                }
            }

            _ => {}
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.view()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        connection::connect().map(|message| GUIMessage::Event(message))
    }
}

fn main() -> iced::Result {
    VaultState::run(Settings::default())
}
