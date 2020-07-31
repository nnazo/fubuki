use crate::{app::Message, ui::style};
use iced::{Container, Element, Length, Row};

mod settings;
pub use settings::*;

mod current_media;
pub use current_media::*;

#[derive(Debug, Clone)]
pub enum Page {
    CurrentMedia,
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Page::CurrentMedia
    }
}

#[derive(Debug, Clone, Default)]
pub struct PageContainer {
    pub page: Page,
    pub current_media: CurrentMediaPage,
    pub settings: SettingsPage,
}

impl PageContainer {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        match self.page {
            Page::CurrentMedia => self.current_media.view(),
            Page::Settings => self.settings.view(),
        }
    }

    pub fn change_page(&mut self, page: Page) {
        self.page = page;
    }

    fn container(content: Element<Message>) -> Container<Message> {
        Container::new(Row::new().push(content))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(style::Container::Background)
    }
}
