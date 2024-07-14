use boring_derive::From;
use std::{collections::BTreeMap, str::FromStr};

use crate::{
    config::{
        client_config::ClientConfig,
        internal_config::{BaseConfig, InternalConfig},
    },
    gui::{
        connection,
        entry::EntryMessage,
        gui_message::GUIMessage,
        state::{entry::EntryState, new_entry::NewEntryState, password::PasswordState},
        temp_message::TempMessage,
        vault::{Vault, VaultMessage},
        INPUT_ID, SHORTCUTS, THEMES,
    },
    info::Info,
    manager_message::ManagerMessage,
    output::Output,
    reads::Reads,
    store::{Store, StoreChoice},
    Password,
};
use iced::{
    alignment, keyboard, theme,
    widget::{self, button, column, container, row, scrollable, text, text_input},
    window, Application, Border, Command, Element, Length, Subscription, Theme,
};
use iced_aw::{
    floating_element,
    menu::{self, Item, Menu, StyleSheet},
    menu_bar, menu_items, modal,
    style::MenuBarStyle,
};
use iced_futures::MaybeSend;
use pants_gen::password::PasswordSpec;
use secrecy::{ExposeSecret, Secret};

use super::prompt::PromptState;

pub struct ManagerState {
    config: ClientConfig,
    info: Info,
    vaults: BTreeMap<String, Vault>,
    internal_state: Vec<InternalState>,
    temp_message: TempMessage,
    stored_clipboard: Option<Password>,
    state: ConnectionState,
    notice: Option<String>,
}

impl Default for ManagerState {
    fn default() -> Self {
        let config: ClientConfig = <ClientConfig as BaseConfig>::load_err();
        Self {
            config,
            info: Info::default(),
            vaults: BTreeMap::new(),
            internal_state: Vec::new(),
            temp_message: TempMessage::default(),
            stored_clipboard: None,
            state: ConnectionState::Disconnected,
            notice: None,
        }
    }
}

enum ConnectionState {
    Disconnected,
    Connected(connection::Connection),
}

impl ManagerState {
    fn get_password(&self) -> Option<Password> {
        for state in &self.internal_state {
            if let InternalState::Password(p) = state {
                return Some(p.password.clone());
            }
        }
        None
    }
    fn handle_password_submit(&mut self, password: Password) -> Command<GUIMessage> {
        let (command, messages) = match &self.temp_message {
            TempMessage::Get(vault, key) => {
                let message = self.temp_message.with_password(password);
                self.internal_state.push(
                    EntryState::from_entry(
                        vault.to_string(),
                        key.to_string(),
                        self.info.get(vault).unwrap().get(key).unwrap().to_string(),
                    )
                    .into(),
                );
                self.temp_message = TempMessage::Update(
                    vault.into(),
                    key.to_string(),
                    StoreChoice::default(),
                    StoreChoice::default().convert_default().as_hash(),
                );
                (text_input::focus(INPUT_ID.clone()), vec![message])
            }
            TempMessage::Delete(..) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                (Command::none(), vec![message, ManagerMessage::Info])
            }
            TempMessage::New(..) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                (Command::none(), vec![message, ManagerMessage::Info])
            }
            TempMessage::Update(..) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                (Command::none(), vec![message, ManagerMessage::Info])
            }
            TempMessage::Empty => {
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                (Command::none(), vec![])
            }
            TempMessage::DeleteVault(..) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                (Command::none(), vec![message, ManagerMessage::Info])
            }
            // non-sense
            TempMessage::DeleteEmptyVault(..) => {
                let message = self.temp_message.with_password(password);
                self.internal_state = vec![];
                self.temp_message = TempMessage::default();
                (Command::none(), vec![message, ManagerMessage::Info])
            }
        };
        self.send_message(messages);
        command
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
    fn update(&mut self, info: Info) {
        let mut vaults = BTreeMap::new();
        for (name, schema) in info.data.iter() {
            let mut vault = Vault::new(name.into(), BTreeMap::new());
            vault.update(schema);
            if let Some(curr_vault) = self.vaults.get(name) {
                vault.expanded = curr_vault.expanded;
            }
            vaults.insert(name.into(), vault);
        }

        self.vaults = vaults;
        self.info = info;
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
        if let TempMessage::Update(_, update_key, ref mut choice, ref mut update_value) =
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

    fn send_message(&mut self, messages: Vec<ManagerMessage>) {
        for message in messages {
            match self.state {
                ConnectionState::Disconnected => {}
                ConnectionState::Connected(ref mut connection) => {
                    connection.send(message);
                }
            }
        }
    }

    fn get_theme(&self) -> Theme {
        THEMES.get(&self.config.theme).cloned().unwrap_or_default()
    }

    fn push_internal_state(&mut self, state: impl Into<InternalState>) -> Command<GUIMessage> {
        self.internal_state.push(state.into());
        text_input::focus(INPUT_ID.clone())
    }

    fn view(&self) -> Element<GUIMessage> {
        let top_layer = self.internal_state.last().map(|state| state.view());

        let menu = |items| Menu::new(items).max_width(180.0).offset(0.0).spacing(0.0);
        let themes: Vec<Item<GUIMessage, iced::Theme, iced::Renderer>> = THEMES
            .iter()
            .map(|(n, t)| {
                let item = if *n == self.config.theme {
                    action_selected_item(text(n), GUIMessage::ChangeTheme(t.clone()))
                } else {
                    action_item(text(n), GUIMessage::ChangeTheme(t.clone()))
                };
                Item::new(item)
            })
            .collect::<Vec<_>>();

        let theme_menu = Menu::new(themes).max_width(200.0).offset(15.0).spacing(5.0);
        #[rustfmt::skip]
        let menu = menu_bar!(
            (section_header("File"), menu(menu_items!(
                (action_item_shortcut("New Vault".to_string()))
                (action_item_shortcut("Quit".to_string()))
                )
            ))
            (section_header("Config"), menu(menu_items!(
                (submenu_item("Theme"), theme_menu)))
            )
        )
        .draw_path(menu::DrawPath::Backdrop)
        .style(|theme:&iced::Theme| menu::Appearance{
            // path_border: Border{
            //     radius: [6.0; 4].into(),
            //     ..Default::default()
            // },
            ..theme.appearance(&MenuBarStyle::Default)
        });

        // let new_vault = button("New Vault").on_press(GUIMessage::NewVault);
        let content = scrollable(
            column(self.vaults.values().map(|v| {
                container(
                    v.view()
                        .map(move |message| GUIMessage::VaultMessage(message, v.name.clone())),
                )
                .padding(3)
                .into()
            }))
            .padding(10),
        );

        // let info = self.temp_message.view();
        let primary = container(column![menu, content]);
        let main = modal(primary, top_layer)
            .backdrop(GUIMessage::Exit)
            .on_esc(GUIMessage::Exit)
            .align_y(alignment::Vertical::Center);
        if let Some(t) = &self.notice {
            let popup = button(
                container(text(t))
                    .width(200.0)
                    .height(100.0)
                    .padding(5)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Appearance {
                            border: Border {
                                color: palette.background.weak.color,
                                width: 4.0,
                                radius: 4.0.into(),
                            },
                            background: Some(palette.background.weak.color.into()),
                            ..Default::default()
                        }
                    }),
            )
            .style(theme::Button::Text)
            .on_press(GUIMessage::ClosePopup);
            floating_element(main, popup)
                .anchor(floating_element::Anchor::NorthEast)
                .hide(false)
                .into()
        } else {
            main.into()
        }
    }
}

#[derive(Debug, From)]
enum InternalState {
    Password(PasswordState),
    Entry(EntryState),
    New(NewEntryState),
    Prompt(PromptState),
    // NewVault(NewVaultState),
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

fn close_popup() -> Command<GUIMessage> {
    delayed_command(5, |_| GUIMessage::ClosePopup)
}

impl InternalState {
    fn view(&self) -> Element<GUIMessage> {
        match self {
            Self::Password(password_state) => password_state.view(),
            Self::New(new_state) => new_state.view(),
            Self::Entry(entry_state) => entry_state.view(),
            Self::Prompt(prompt_state) => prompt_state.view(),
            // Self::NewVault(new_vault_state) => new_vault_state.view(),
        }
    }
}

impl Application for ManagerState {
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
                    self.send_message(vec![ManagerMessage::Info]);
                }
                connection::Event::Disconnected => {
                    self.state = ConnectionState::Disconnected;
                }
                connection::Event::ReceiveOutput(output) => match output {
                    Output::Info(info) => {
                        // println!("Received info: {:?}", info);
                        self.update(info);
                    }
                    Output::Read(value) => {
                        // println!("Received read: {:?}", value);
                        self.update_entry(value);
                    }
                    Output::Nothing => {}
                    _ => todo!(),
                },

                connection::Event::ReceiveError(e) => {
                    self.internal_state = vec![];
                    self.temp_message = TempMessage::default();
                    self.notice = Some(format!("Encountered an error: {}", e));
                    return close_popup();
                }
            },
            GUIMessage::ClosePopup => {
                self.notice = None;
            }
            // GUIMessage::Send(message) => self.send_message(vec![message]),
            GUIMessage::VaultMessage(message, vault) => match message {
                VaultMessage::Entry(entry_message, key) => match entry_message {
                    EntryMessage::Delete => {
                        self.temp_message = TempMessage::Delete(vault, key);
                        if self.needs_password() {
                            return self.push_internal_state(PasswordState::default());
                        }
                    }
                    EntryMessage::View => {
                        self.temp_message = TempMessage::Get(vault, key.clone());

                        if self.needs_password() {
                            return self.push_internal_state(PasswordState::default());
                        }
                    }
                },
                VaultMessage::NewEntry => {
                    self.temp_message = TempMessage::New(
                        vault.to_string(),
                        String::new(),
                        StoreChoice::default(),
                        StoreChoice::default().convert_default().as_hash(),
                    );
                    let command = self.push_internal_state(NewEntryState::for_vault(vault));
                    let gen_password = delayed_command(0, |_| GUIMessage::GeneratePassword);
                    return Command::batch(vec![command, gen_password]);
                }
                VaultMessage::Delete => {
                    if self.info.get(&vault).unwrap().is_empty() {
                        self.send_message(vec![
                            ManagerMessage::DeleteEmptyVault(vault),
                            ManagerMessage::Info,
                        ]);
                    } else {
                        self.temp_message = TempMessage::DeleteVault(vault);
                    }
                    if self.needs_password() {
                        return self.push_internal_state(PasswordState::default());
                    }
                }
                VaultMessage::Toggle => {
                    if let Some(value) = self.vaults.get_mut(&vault) {
                        value.toggle();
                    }
                }
            },

            GUIMessage::PasswordChanged(p) => {
                if let Some(InternalState::Password(password_state)) = self.active_state_mut() {
                    password_state.password = p;
                }
            }
            GUIMessage::PasswordConfirmChanged(p) => {
                if let Some(InternalState::Password(password_state)) = self.active_state_mut() {
                    password_state.confirm = Some(p);
                }
            }
            GUIMessage::PromptChanged(p) => {
                if let Some(InternalState::Prompt(prompt_state)) = self.active_state_mut() {
                    prompt_state.vault = p;
                }
            }

            GUIMessage::ChangeName(n) => {
                if let Some(InternalState::New(new_state)) = self.active_state_mut() {
                    new_state.name.clone_from(&n);
                }
                if let TempMessage::New(_, ref mut key, _, _) = &mut self.temp_message {
                    *key = n;
                }
            }
            GUIMessage::SelectStyle(choice) => {
                if let Some(InternalState::New(new_state)) = self.active_state_mut() {
                    new_state.choice = choice;
                    // new_state.value = choice.convert_default().as_hash();
                }
                if let TempMessage::New(_, _, ref mut style, _) = &mut self.temp_message {
                    *style = choice;
                    // *value = choice.convert_default().as_hash();
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
                    TempMessage::New(_, _, _, ref mut value) => {
                        value.insert(k, v);
                    }
                    TempMessage::Update(_, _, _, ref mut value) => {
                        value.insert(k, v);
                    }
                    _ => {}
                };
            }
            GUIMessage::GeneratePassword => {
                let spec = PasswordSpec::from_str(&self.config.password_spec).unwrap();
                let password: Secret<String> = spec.generate().unwrap().into();
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
                    TempMessage::New(_, _, _, ref mut value) => {
                        value.insert("password".to_string(), password);
                    }
                    TempMessage::Update(_, _, _, ref mut value) => {
                        value.insert("password".to_string(), password);
                    }
                    _ => {}
                };
            }

            GUIMessage::Submit => {
                if let Some(active_state) = self.active_state() {
                    match active_state {
                        InternalState::Password(password_state) => {
                            if password_state.valid() {
                                return self
                                    .handle_password_submit(password_state.password.clone());
                            } else {
                                self.notice = Some("Passwords do not match".into());
                                return close_popup();
                            }
                        }
                        InternalState::New(new_state) => {
                            if let Some(schema) = self.info.get(&new_state.vault) {
                                // println!("{:?}", schema);
                                if self.temp_message.complete() {
                                    if schema.is_empty() {
                                        return self.push_internal_state(PasswordState::confirm());
                                    } else if !schema.data.contains_key(&new_state.name) {
                                        return self.push_internal_state(PasswordState::default());
                                    }
                                } else {
                                    self.notice = Some("Fill all fields before submitting".into());
                                    return close_popup();
                                }
                            }
                        }
                        InternalState::Entry(entry_state) => {
                            if let Some(schema) = self.info.get(&entry_state.vault) {
                                if schema.data.contains_key(&entry_state.key)
                                    && self.temp_message.complete()
                                {
                                    if let Some(password) = self.get_password() {
                                        let message = self.temp_message.with_password(password);
                                        self.send_message(vec![message]);
                                        self.temp_message = TempMessage::default();
                                        self.internal_state = vec![];
                                    } else {
                                        return self.push_internal_state(PasswordState::default());
                                    }
                                } else {
                                    self.notice = Some("Fill all fields before submitting".into());
                                    return close_popup();
                                }
                            }
                        }
                        // InternalState::NewVault(new_vault_state) => {
                        //     if !self.info.data.contains_key(&new_vault_state.vault)
                        //         && self.temp_message.complete()
                        //     {
                        //         let new_message =
                        //             ManagerMessage::NewVault(new_vault_state.vault.clone());
                        //         self.internal_state.push(PasswordState::default().into());
                        //         self.send_message(vec![new_message])
                        //     }
                        // }
                        InternalState::Prompt(prompt_state) => {
                            if !self.info.data.contains_key(&prompt_state.vault)
                                && !prompt_state.vault.is_empty()
                            {
                                let message = ManagerMessage::NewVault(prompt_state.vault.clone());
                                self.send_message(vec![message, ManagerMessage::Info]);
                                self.internal_state.pop();
                            } else {
                                if self.info.data.contains_key(&prompt_state.vault) {
                                    self.notice = Some("This vault already exists".into());
                                    return close_popup();
                                }
                                if prompt_state.vault.is_empty() {
                                    self.notice = Some("Need a name to create vault".into());
                                    return close_popup();
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
                                TempMessage::Delete(..) => {
                                    self.temp_message = TempMessage::default();
                                }
                                TempMessage::Get(..) => {
                                    self.temp_message = TempMessage::default();
                                }
                                TempMessage::DeleteVault(..) => {
                                    self.temp_message = TempMessage::default();
                                }
                                TempMessage::DeleteEmptyVault(..) => {
                                    self.temp_message = TempMessage::default();
                                }
                                TempMessage::Update(..) => {}
                                TempMessage::New(..) => {}
                                TempMessage::Empty => {}
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
                        InternalState::Prompt(_) => {
                            self.internal_state.pop();
                        }
                    }
                }
            }

            GUIMessage::ShowPassword => {
                if let Some(state) = self.active_state_mut() {
                    match state {
                        InternalState::Entry(entry_state) => entry_state.hidden = false,
                        InternalState::New(new_state) => new_state.hidden = false,
                        _ => (),
                    }
                }
            }
            GUIMessage::HidePassword => {
                if let Some(state) = self.active_state_mut() {
                    match state {
                        InternalState::Entry(entry_state) => entry_state.hidden = true,
                        InternalState::New(new_state) => new_state.hidden = true,
                        _ => (),
                    }
                }
            }
            GUIMessage::CopyPassword => {
                if let Some(InternalState::Entry(entry_state)) = self.active_state_mut() {
                    if let Some(p) = entry_state.get_password() {
                        return Command::batch(vec![
                            iced::clipboard::read(|s| {
                                GUIMessage::CopyClipboard(s.map(|x| x.into()))
                            }),
                            iced::clipboard::write(p.expose_secret().into()),
                            delayed_command(self.config.clipboard_time, |_| {
                                GUIMessage::ClearClipboard
                            }),
                        ]);
                    }
                }
            }
            GUIMessage::CopyClipboard(data) => self.stored_clipboard = data,
            GUIMessage::ClearClipboard => {
                let contents: Secret<String> = self
                    .stored_clipboard
                    .clone()
                    .unwrap_or_else(|| Secret::new(String::new()));
                self.stored_clipboard = None;
                return iced::clipboard::write(contents.expose_secret().into());
            }
            GUIMessage::NewVault => return self.push_internal_state(PromptState::default()),
            GUIMessage::ChangeTheme(theme) => {
                self.config.theme = theme.to_string();
                if self.config.save().is_err() {
                    self.notice = Some("Failed to save config file".into());
                    return close_popup();
                }
            }
            GUIMessage::TabPressed(shift) => {
                return if shift {
                    widget::focus_previous()
                } else {
                    widget::focus_next()
                }
            }
            GUIMessage::Close => return window::close(window::Id::MAIN),
            GUIMessage::Nothing => {}
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.view()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let connection_subscriber = connection::connect().map(GUIMessage::Event);

        let keyboard_subscriber = keyboard::on_key_press(|key, modifiers| {
            for (_, shortcut) in SHORTCUTS.iter() {
                let res = shortcut.check(&key, &modifiers);
                if res.is_some() {
                    return res;
                }
            }
            None
        });

        // let keyboard_subscriber = keyboard::on_key_press(|key, modifiers| {
        //     // println!("{:?}, {:?}", key, modifiers);
        //     println!("{:?}", keyboard::Modifiers::COMMAND);
        //     match (key.as_ref(), modifiers) {
        //         (key::Key::Character("n"), keyboard::Modifiers::COMMAND) => {
        //             Some(GUIMessage::NewVault)
        //         }
        //         (key::Key::Character("q"), keyboard::Modifiers::COMMAND) => Some(GUIMessage::Close),
        //         (key::Key::Named(key::Named::Tab), _) => {
        //             Some(GUIMessage::TabPressed(modifiers.shift()))
        //         }
        //         _ => {
        //             // println!("{:?}, {:?}", key, modifiers);
        //             None
        //         }
        //     }
        //     // let keyboard::Key::Named(key) = key else {
        //     //     return None;
        //     // };
        //     //
        //     // match (key, modifiers) {
        //     //     (key::Named::Tab, _) => Some(Message::TabPressed {
        //     //         shift: modifiers.shift(),
        //     //     }),
        //     //     (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
        //     //         Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
        //     //     }
        //     //     (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
        //     //         Some(Message::ToggleFullscreen(window::Mode::Windowed))
        //     //     }
        //     //     _ => None,
        //     // }
        // });

        Subscription::batch(vec![connection_subscriber, keyboard_subscriber])
    }

    fn theme(&self) -> Theme {
        self.get_theme()
    }
}

fn section_header<'a>(label: &str) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
    base_button(text(label), Some(GUIMessage::Nothing))
}

fn submenu_item<'a>(label: &str) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
    base_button(
        row![
            text(label).width(Length::Fill),
            text(">").width(Length::Shrink)
        ],
        Some(GUIMessage::Nothing),
    )
    .width(Length::Fill)
}

// fn menu_item<'a>(label: &str) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
//     base_button(text(label), None).width(Length::Fill)
// }

fn action_item<'a>(
    label: impl Into<Element<'a, GUIMessage, iced::Theme, iced::Renderer>>,
    message: GUIMessage,
) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
    base_button(label, Some(message)).width(Length::Fill)
}

fn action_item_shortcut<'a>(
    name: String,
) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
    if let Some(shortcut) = SHORTCUTS.get(&name) {
        base_button(
            row![
                container(text(name)).width(Length::Fill),
                text(shortcut.key_display())
            ],
            Some(shortcut.message().clone()),
        )
    } else {
        base_button(text(name), None)
    }
}

fn action_selected_item<'a>(
    label: impl Into<Element<'a, GUIMessage, iced::Theme, iced::Renderer>>,
    message: GUIMessage,
) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
    base_button(
        row![
            container(label).width(Length::Fill),
            text("<").width(Length::Shrink)
        ],
        Some(message),
    )
}

fn base_button<'a>(
    content: impl Into<Element<'a, GUIMessage, iced::Theme, iced::Renderer>>,
    msg: Option<GUIMessage>,
) -> button::Button<'a, GUIMessage, iced::Theme, iced::Renderer> {
    let button = if let Some(msg) = msg {
        button(content).padding([4, 8]).on_press(msg)
    } else {
        button(content).padding([4, 8])
    };
    button.style(theme::Button::Secondary)
}
