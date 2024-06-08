use std::str::FromStr;

use crate::{
    config::ClientConfig,
    gui::{
        connection,
        entry::{Entry, EntryMessage},
        gui_message::GUIMessage,
        state::{entry::EntryState, new_entry::NewEntryState, password::PasswordState},
        temp_message::TempMessage,
    },
    message::Message,
    reads::Reads,
    schema::Schema,
    store::{Store, StoreChoice},
};
use iced::{
    alignment,
    widget::{button, column, container, scrollable, text},
    Application, Command, Element, Length, Subscription, Theme,
};
use iced_aw::modal;
use iced_futures::MaybeSend;
use pants_gen::password::PasswordSpec;

pub struct VaultState {
    config: ClientConfig,
    schema: Schema,
    entries: Vec<Entry>,
    internal_state: Vec<InternalState>,
    temp_message: TempMessage,
    stored_clipboard: Option<String>,
    state: ConnectionState,
}

impl Default for VaultState {
    fn default() -> Self {
        let config: ClientConfig = ClientConfig::figment().extract().unwrap();
        Self {
            config,
            schema: Schema::default(),
            entries: vec![],
            internal_state: Vec::new(),
            temp_message: TempMessage::default(),
            stored_clipboard: None,
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

fn delayed_command(
    time: u64,
    callback: impl FnOnce(()) -> GUIMessage + 'static + MaybeSend,
) -> Command<GUIMessage> {
    Command::perform(
        async move {
            let _ = async_std::task::sleep(std::time::Duration::from_secs(time)).await;
            
        },
        callback,
    )
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
                    new_state.name.clone_from(&n);
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
                let spec = PasswordSpec::from_str(&self.config.password_spec).unwrap();
                let password = spec.generate().unwrap();
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
                        return Command::batch(vec![
                            iced::clipboard::read(GUIMessage::CopyClipboard),
                            iced::clipboard::write(p),
                            delayed_command(self.config.clipboard_time, |_| {
                                GUIMessage::ClearClipboard
                            }),
                        ]);
                    }
                }
            }
            GUIMessage::CopyClipboard(data) => self.stored_clipboard = data,
            GUIMessage::ClearClipboard => {
                let contents = self.stored_clipboard.clone().unwrap_or_default();
                self.stored_clipboard = None;
                return iced::clipboard::write(contents);
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
