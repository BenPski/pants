use std::collections::HashMap;

use iced::{
    alignment, theme,
    widget::{button, column, container, pick_list, row, scrollable, text, text_input},
    Application, Command, Element, Length, Settings, Subscription, Theme,
};
use iced_aw::{modal, Card};
use pants_gen::password::Password;
use pants_store::{
    gui::connection,
    message::Message,
    reads::Reads,
    schema::Schema,
    store::{Store, StoreChoice},
};

#[derive(Debug, Clone)]
enum GUIMessage {
    Exit,
    Submit,
    EntryMessage(EntryMessage, String),
    ShowPassword,
    HidePassword,
    CopyPassword,
    PasswordChanged(String),
    NewEntry,
    ChangeName(String),
    SelectStyle(StoreChoice),
    UpdateField(String, String),
    GeneratePassword,
    Event(connection::Event),
    // Send(Message),
}

struct VaultState {
    schema: Schema,
    entries: Vec<Entry>,
    internal_state: Vec<InternalState>,
    temp_message: TempMessage,
    state: ConnectionState,
}

impl Default for VaultState {
    fn default() -> Self {
        Self {
            schema: Schema::default(),
            entries: vec![],
            internal_state: Vec::new(),
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
    fn get_password(&self) -> Option<String> {
        for state in &self.internal_state {
            if let InternalState::Password(p) = state {
                return Some(p.password.clone());
            }
        }
        None
    }
    fn handle_password_submit(&mut self, password: String) {
        let messages = match &self.temp_message {
            TempMessage::Get(key) => {
                let message = self.temp_message.with_password(password);
                self.internal_state.push(
                    EntryState::from_entry(
                        key.to_string(),
                        self.schema.get(key).unwrap().to_string(),
                    )
                    .into(),
                );
                self.temp_message = TempMessage::Update(
                    key.to_string(),
                    StoreChoice::default(),
                    StoreChoice::default().convert_default().as_hash(),
                );
                vec![message]
            }
            TempMessage::Delete(_) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                vec![message, Message::Schema]
            }
            TempMessage::New(_, _, _) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                vec![message, Message::Schema]
            }
            TempMessage::Update(_, _, _) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                vec![message, Message::Schema]
            }
            TempMessage::Empty => {
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                vec![]
            }
        };
        self.send_message(messages);
    }
    fn active_state(&self) -> Option<&InternalState> {
        self.internal_state.last()
    }
    fn active_state_mut(&mut self) -> Option<&mut InternalState> {
        self.internal_state.last_mut()
    }
    // fn reset(&mut self) {
    //     self.temp_message = TempMessage::default();
    //     self.entry_state = None;
    // }
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
        // TODO: check if robust, could be that a response was given to a lower down state, but I
        // find it unlikely it will get to be that way
        if let Some(InternalState::Entry(entry)) = self.active_state_mut() {
            for (key, value) in &data.data {
                if *key == entry.key {
                    entry.update(value.clone());
                }
            }
        }
        // seems dumb to loop twice
        if let TempMessage::Update(update_key, ref mut choice, ref mut update_value) =
            &mut self.temp_message
        {
            for (key, value) in data.data {
                if *update_key == key {
                    let (new_choice, new_values) = value.split();
                    *choice = new_choice;
                    *update_value = new_values;
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
        let top_layer = self.internal_state.last().map(|state| state.view());

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
        let primary = container(column![header, content, new_button, info]);
        modal(primary, top_layer)
            .backdrop(GUIMessage::Exit)
            .on_esc(GUIMessage::Exit)
            .align_y(alignment::Vertical::Center)
            .into()
    }
}

#[derive(Debug, Clone, Default)]
enum TempMessage {
    #[default]
    Empty,
    Delete(String),
    Get(String),
    New(String, StoreChoice, HashMap<String, String>),
    Update(String, StoreChoice, HashMap<String, String>),
}

impl TempMessage {
    fn needs_password(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::Delete(_) => true,
            Self::Get(_) => true,
            Self::New(_, _, _) => true,
            Self::Update(_, _, _) => true,
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
            Self::Update(name, _, fields) => {
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

    fn with_password(&self, password: String) -> Message {
        match self {
            Self::Delete(key) => Message::Delete(password, key.to_string()),
            Self::Get(key) => Message::Get(password, key.to_string()),
            Self::New(key, choice, value) => {
                Message::Update(password, key.clone(), choice.convert(value).unwrap())
            }
            Self::Update(key, choice, value) => {
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
            TempMessage::Update(key, _, _) => {
                let info = text(format!("Working on updating entry {}", key));
                container(info).into()
            }
            Self::Empty => {
                let info = text("Working on nothing".to_string());
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
        let header = text("New entry");
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
                let password_input = text_input("Password", self.value.get("password").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("password".to_string(), v));
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);

                container(row![prefix, password_input, password_generate])
            }
            StoreChoice::UsernamePassword => {
                let username_prefix = text("Username:");
                let password_prefix = text("Password:");
                let username_input = text_input("Username", self.value.get("username").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("username".to_string(), v));
                let password_input = text_input("Password", self.value.get("password").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("password".to_string(), v));

                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(column![
                    row![username_prefix, username_input],
                    row![password_prefix, password_input, password_generate]
                ])
            }
        };
        let create_button = button("Create").on_press(GUIMessage::Submit);
        let cancel_button = button("Cancel").on_press(GUIMessage::Exit);
        Card::new(
            header,
            container(column![
                row![name_prefix, name_input],
                style_choice,
                data_input,
                row![create_button, cancel_button]
            ]),
        )
        .max_width(500.0)
        .into()
    }
}

#[derive(Debug, Clone)]
struct EntryState {
    key: String,
    choice: StoreChoice,
    value: HashMap<String, String>,
    hidden: bool,
}

impl EntryState {
    fn view(&self) -> Element<GUIMessage> {
        let header = text(self.key.clone());
        let data_input = match &self.choice {
            StoreChoice::Password => {
                let prefix = text("Password:");
                let password_input = text_input("Password", self.value.get("password").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("password".to_string(), v))
                    .secure(self.hidden);
                let show_button = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                let copy_button = button("Copy").on_press(GUIMessage::CopyPassword);
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(row![
                    prefix,
                    password_input,
                    password_generate,
                    copy_button,
                    show_button
                ])
            }
            StoreChoice::UsernamePassword => {
                let username_prefix = text("Username:");
                let password_prefix = text("Password:");
                let username_input = text_input("Username", self.value.get("username").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("username".to_string(), v));
                let password_input = text_input("Password", self.value.get("password").unwrap())
                    .width(Length::Fill)
                    .on_input(|v| GUIMessage::UpdateField("password".to_string(), v))
                    .secure(self.hidden);
                let show_button = if self.hidden {
                    button("Show").on_press(GUIMessage::ShowPassword)
                } else {
                    button("Hide").on_press(GUIMessage::HidePassword)
                };
                let copy_button = button("Copy").on_press(GUIMessage::CopyPassword);
                let password_generate = button("Generate").on_press(GUIMessage::GeneratePassword);
                container(column![
                    row![username_prefix, username_input],
                    row![
                        password_prefix,
                        password_input,
                        password_generate,
                        copy_button,
                        show_button
                    ]
                ])
            }
        };

        let save_button = button("Save").on_press(GUIMessage::Submit);
        let done_button = button("Done").on_press(GUIMessage::Exit);
        Card::new(
            header,
            container(column![data_input, row![save_button, done_button]]),
        )
        .max_width(500.0)
        .into()
    }

    fn update(&mut self, value: Store) {
        let (choice, value) = value.split();
        self.choice = choice;
        self.value = value;
    }

    fn get_password(&self) -> Option<String> {
        for (key, value) in self.value.iter() {
            if key == "password" {
                return Some(value.to_string());
            }
        }
        None
    }

    fn from_entry(key: String, style: String) -> Self {
        let value = match style.as_str() {
            "password" => Store::Password(String::new()),
            "username-password" => Store::UsernamePassword(String::new(), String::new()),
            _ => todo!(),
        };
        let (choice, value) = value.split();
        EntryState {
            key,
            choice,
            value,
            hidden: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct PasswordState {
    password: String,
}

impl PasswordState {
    fn view(&self) -> Element<GUIMessage> {
        let header = text("Vault password");
        let password_input = text_input("vault password", &self.password.clone())
            .on_input(GUIMessage::PasswordChanged)
            .on_submit(GUIMessage::Submit)
            .width(Length::Fill)
            .secure(true);
        let cancel = button("Cancel").on_press(GUIMessage::Exit);
        Card::new(header, container(column![password_input, cancel]))
            .max_width(500.0)
            .into()
    }
}

#[derive(Debug)]
enum InternalState {
    Password(PasswordState),
    Entry(EntryState),
    New(NewEntryState),
}

impl From<PasswordState> for InternalState {
    fn from(value: PasswordState) -> Self {
        InternalState::Password(value)
    }
}

impl From<EntryState> for InternalState {
    fn from(value: EntryState) -> Self {
        InternalState::Entry(value)
    }
}

impl From<NewEntryState> for InternalState {
    fn from(value: NewEntryState) -> Self {
        InternalState::New(value)
    }
}

impl InternalState {
    fn view(&self) -> Element<GUIMessage> {
        match self {
            Self::Password(password_state) => password_state.view(),
            Self::New(new_state) => new_state.view(),
            Self::Entry(entry_state) => entry_state.view(),
        }
    }
}

#[derive(Debug, Clone)]
struct Entry {
    key: String,
}

#[derive(Debug, Clone)]
enum EntryMessage {
    Delete,
    View,
}

impl Entry {
    fn new(key: String, _style: String) -> Self {
        Entry { key }
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
            // GUIMessage::Send(message) => self.send_message(vec![message]),
            GUIMessage::EntryMessage(EntryMessage::Delete, key) => {
                self.temp_message = TempMessage::Delete(key);
                if self.needs_password() {
                    self.internal_state.push(PasswordState::default().into());
                }
            }
            GUIMessage::EntryMessage(EntryMessage::View, key) => {
                self.temp_message = TempMessage::Get(key.clone());

                if self.needs_password() {
                    self.internal_state.push(PasswordState::default().into());
                }
            }
            GUIMessage::PasswordChanged(p) => {
                if let Some(InternalState::Password(password_state)) = self.active_state_mut() {
                    password_state.password = p;
                }
            }

            GUIMessage::ChangeName(n) => {
                if let Some(InternalState::New(new_state)) = self.active_state_mut() {
                    new_state.name = n.clone();
                }
                if let TempMessage::New(ref mut key, _, _) = &mut self.temp_message {
                    *key = n;
                }
            }
            GUIMessage::SelectStyle(choice) => {
                if let Some(InternalState::New(new_state)) = self.active_state_mut() {
                    new_state.choice = choice;
                    new_state.value = choice.convert_default().as_hash();
                }
                if let TempMessage::New(_, ref mut style, ref mut value) = &mut self.temp_message {
                    *style = choice;
                    *value = choice.convert_default().as_hash();
                }
            }
            GUIMessage::UpdateField(k, v) => {
                match self.active_state_mut() {
                    Some(InternalState::New(new_state)) => {
                        new_state.value.insert(k.clone(), v.clone());
                    }
                    Some(InternalState::Entry(entry_state)) => {
                        entry_state.value.insert(k.clone(), v.clone());
                    }
                    _ => {}
                };
                match &mut self.temp_message {
                    TempMessage::New(_, _, ref mut value) => {
                        value.insert(k, v);
                    }
                    TempMessage::Update(_, _, ref mut value) => {
                        value.insert(k, v);
                    }
                    _ => {}
                };
            }
            GUIMessage::GeneratePassword => {
                let password = Password::default().generate().unwrap();
                match self.active_state_mut() {
                    Some(InternalState::New(new_state)) => {
                        new_state
                            .value
                            .insert("password".to_string(), password.clone());
                    }
                    Some(InternalState::Entry(entry_state)) => {
                        entry_state
                            .value
                            .insert("password".to_string(), password.clone());
                    }
                    _ => {}
                };
                match &mut self.temp_message {
                    TempMessage::New(_, _, ref mut value) => {
                        value.insert("password".to_string(), password);
                    }
                    TempMessage::Update(_, _, ref mut value) => {
                        value.insert("password".to_string(), password);
                    }
                    _ => {}
                };
            }

            GUIMessage::NewEntry => {
                self.temp_message = TempMessage::New(
                    String::new(),
                    StoreChoice::default(),
                    StoreChoice::default().convert_default().as_hash(),
                );
                self.internal_state.push(NewEntryState::default().into());
            }
            GUIMessage::Submit => {
                if let Some(active_state) = self.active_state() {
                    match active_state {
                        InternalState::Password(password_state) => {
                            self.handle_password_submit(password_state.password.clone());
                        }
                        InternalState::New(new_state) => {
                            if !self.schema.data.contains_key(&new_state.name)
                                && self.temp_message.complete()
                            {
                                self.internal_state.push(PasswordState::default().into());
                            }
                        }
                        InternalState::Entry(entry_state) => {
                            if self.schema.data.contains_key(&entry_state.key)
                                && self.temp_message.complete()
                            {
                                if let Some(password) = self.get_password() {
                                    let message = self.temp_message.with_password(password);
                                    self.send_message(vec![message]);
                                    self.temp_message = TempMessage::default();
                                    self.internal_state = vec![];
                                } else {
                                    self.internal_state.push(PasswordState::default().into());
                                }
                            }
                        }
                    }
                }
            }
            GUIMessage::Exit => {
                if let Some(active_state) = self.active_state() {
                    match active_state {
                        InternalState::Password(_password_state) => {
                            match self.temp_message {
                                TempMessage::Delete(_) => {
                                    self.temp_message = TempMessage::default();
                                }
                                TempMessage::Get(_) => {
                                    self.temp_message = TempMessage::default();
                                }
                                _ => {}
                            }
                            self.internal_state.pop();
                        }
                        InternalState::Entry(_entry_state) => {
                            self.temp_message = TempMessage::default();
                            self.internal_state = vec![];
                        }
                        InternalState::New(_new_state) => {
                            self.temp_message = TempMessage::default();
                            self.internal_state = vec![];
                        }
                    }
                }
            }

            GUIMessage::ShowPassword => {
                if let Some(InternalState::Entry(entry_state)) = self.active_state_mut() {
                    entry_state.hidden = false;
                }
            }
            GUIMessage::HidePassword => {
                if let Some(InternalState::Entry(entry_state)) = self.active_state_mut() {
                    entry_state.hidden = true;
                }
            }
            GUIMessage::CopyPassword => {
                if let Some(InternalState::Entry(entry_state)) = self.active_state_mut() {
                    if let Some(p) = entry_state.get_password() {
                        return iced::clipboard::write(p);
                    }
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.view()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        connection::connect().map(GUIMessage::Event)
    }
}

fn main() -> iced::Result {
    VaultState::run(Settings::default())
}
