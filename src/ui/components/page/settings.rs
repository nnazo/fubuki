use iced::{Element, Text, Column, button, Button, HorizontalAlignment, Command};
use crate::{ui::style, app::{App, Event, Message}};
use super::PageContainer;

#[derive(Debug, Clone, Default)]
pub struct SettingsPage {
    pub logged_in: bool,
    refresh_list_state: button::State,
    logout_state: button::State,
}

impl SettingsPage {
    pub fn update(&mut self, _msg: Message) {}
    
    pub fn view(&mut self) -> Element<Message> {
        let mut col = Column::new().padding(24).spacing(12);
        let text_size = 14;
        let button_padding = 12;

        if self.logged_in {
            col = col.push(Button::new(
                &mut self.refresh_list_state,
                Text::new("Refresh Lists")
                    .size(text_size)
                    .horizontal_alignment(HorizontalAlignment::Center)
            )
            .padding(button_padding)
            .style(style::Button::Accent)
            .on_press(RefreshLists.into())
            );
    
            col = col.push(Button::new(
                &mut self.logout_state,
                Text::new("Logout")
                    .size(text_size)
                    .horizontal_alignment(HorizontalAlignment::Center)
            )
            .padding(button_padding)
            .style(style::Button::Danger)
            .on_press(Logout.into()));
        } else {
            col = col.push(Button::new(
                &mut self.logout_state,
                Text::new("Login")
                    .size(text_size)
                    .horizontal_alignment(HorizontalAlignment::Center)
            )
            .padding(button_padding)
            .style(style::Button::Accent)
            .on_press(Login.into()));
        }

        PageContainer::container(col.into()).into()
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
            Ok(_) => {},
            Err(err) => {
                eprintln!("could not forget token: {}", err);
            },
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