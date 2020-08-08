use super::PageContainer;
use crate::{
    app::{App, Event, Message},
    ui::style,
};
use iced::{button, Button, Column, Command, Element, HorizontalAlignment, Text, TextInput, text_input, Length, Container, VerticalAlignment};

#[derive(Debug, Clone, Default)]
pub struct SettingsPage {
    pub logged_in: bool,
    refresh_list_state: button::State,
    login_state: button::State,
    update_delay_state: text_input::State,
    update_delay_value: String,
}

impl SettingsPage {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        let mut col = Column::new().spacing(12).push(Self::header_title("AniList"));
        let input_padding = 10;

        let mut anilist_inner = Column::new().spacing(12);

        if self.logged_in {
            anilist_inner = anilist_inner.push(
                Self::button(&mut self.refresh_list_state, "Refresh Lists", style::Button::Accent, RefreshLists.into())
            ).push(
                Self::button(&mut self.login_state, "Logout", style::Button::Danger, Logout.into())
            );
        } else {
            anilist_inner = anilist_inner.push(
                Self::button(&mut self.login_state, "Login", style::Button::Accent, Login.into())
            );
        }
        col = col.push(Self::container(anilist_inner.into()));

        let general_inner = Column::new().spacing(12);
        let mut update_delay = Column::new().spacing(12);
        self.update_delay_value = format!("{}", crate::settings::get_settings().read().unwrap().update_delay);

        update_delay = update_delay.push(
            Text::new("List update delay (seconds)")
            .size(16)
            .horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Center)
        )
        .push(
            TextInput::new(&mut self.update_delay_state, "", &self.update_delay_value, |value| {
                SettingChange::UpdateDelay(value, false).into()
            })
            .style(style::Input)
            .padding(input_padding)
            .width(Length::Units(80))
            .on_submit(SettingChange::UpdateDelay(self.update_delay_value.clone(), true).into())
        );


        col = col
            .push(Self::header_title("General"))
            .push(Self::container(general_inner.push(update_delay).into()));

        PageContainer::container(col.into()).into()
    }

    fn container(element: Element<Message>) -> Element<Message> {
        Container::new(element).padding(12).into()
    }

    fn button<'a>(state: &'a mut button::State, text: &'static str, btn_style: style::Button, msg: Message) -> Element<'a, Message> {
        let button_padding = 12;
        let text_size = 14;
        Button::new(
            state,
            Text::new(text)
                .size(text_size)
                .horizontal_alignment(HorizontalAlignment::Center),
        )
        .padding(button_padding)
        .style(btn_style)
        .on_press(msg)
        .into()
    }
    
    fn header_title(text: &str) -> Element<Message> {
        let text_size = 18;
        Container::new(
            Text::new(text).size(text_size).horizontal_alignment(HorizontalAlignment::Left)
        )
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct RefreshLists;

impl Event for RefreshLists {
    fn handle(self, app: &mut App) -> Command<Message> {
        let settings = crate::settings::SETTINGS.read().unwrap();
        let token = settings.anilist.token().clone();
        if let Some(token) = token {
            if let Some(user) = &app.user {
                return App::query_user_lists(token, user.id);
            }
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Logout;

impl Event for Logout {
    fn handle(self, app: &mut App) -> Command<Message> {
        let mut settings = crate::settings::SETTINGS.write().unwrap();
        match settings.anilist.forget_token() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("could not forget token: {}", err);
            }
        };
        app.user = None;
        app.anime_list = None;
        app.manga_list = None;
        app.nav.set_avatar(None);
        app.page.settings.logged_in = false;
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Login;

impl Event for Login {
    fn handle(self, app: &mut App) -> Command<Message> {
        app.page.settings.logged_in = true;
        if app.user.is_some() {
            Command::none()
        } else {
            App::auth()
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingChange {
    UpdateDelay(String, bool),
}

impl Event for SettingChange {
    fn handle(self, _app: &mut App) -> Command<Message> {
        let mut settings = crate::settings::get_settings().write().unwrap();
        let mut changed = false;
        match self {
            SettingChange::UpdateDelay(delay, save) => {
                match delay.parse::<u64>() {
                    Ok(delay) => {
                        settings.update_delay = delay;
                        changed = save;
                    },
                    Err(err) => {
                        eprintln!("could not parse new update delay {}", err)
                    }
                }
            }
        }
        if changed {
            if let Err(err) = settings.save() {
                eprintln!("error saving settings: {}", err);
            }
        }
        Command::none()
    }
}