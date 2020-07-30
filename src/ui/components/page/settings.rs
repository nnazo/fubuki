use iced::{Element, Text, Column, button, Button, HorizontalAlignment, Command};
use crate::{ui::style, app::{App, Event, Message}};
use super::PageContainer;

#[derive(Debug, Clone, Default)]
pub struct SettingsPage {
    refresh_list_state: button::State,
}

impl SettingsPage {
    pub fn update(&mut self, _msg: Message) {}
    
    pub fn view(&mut self) -> Element<Message> {
        let mut col = Column::new().padding(24).spacing(12);
        let text_size = 14;
        let button_padding = 12;

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