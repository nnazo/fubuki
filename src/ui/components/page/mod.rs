use crate::{anilist::MediaType, app::Message, ui::style};
use iced::{Container, Element, Length, Row};

mod settings;
pub use settings::*;

mod current_media;
pub use current_media::*;

mod media_list;
pub use media_list::*;

#[derive(Debug, Clone)]
pub enum Page {
    Anime,
    Manga,
    CurrentMedia,
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Page::CurrentMedia
    }
}

#[derive(Debug, Clone)]
pub struct PageContainer {
    pub page: Page,
    pub current_media: CurrentMediaPage,
    pub settings: SettingsPage,
    pub anime: MediaListPage,
    pub manga: MediaListPage,
}

impl PageContainer {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        match self.page {
            Page::CurrentMedia => self.current_media.view(),
            Page::Settings => self.settings.view(),
            Page::Anime => self.anime.view(),
            Page::Manga => self.manga.view(),
        }
    }

    pub fn change_page(&mut self, page: Page) {
        self.page = page;
    }

    fn container(content: Element<Message>) -> Container<Message> {
        Container::new(Row::new().push(content))
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(24)
            .style(style::Container::Background)
    }
}

impl Default for PageContainer {
    fn default() -> Self {
        PageContainer {
            page: Page::default(),
            current_media: CurrentMediaPage::default(),
            settings: SettingsPage::default(),
            anime: MediaListPage::new(MediaType::Anime),
            manga: MediaListPage::new(MediaType::Manga),
        }
    }
}
