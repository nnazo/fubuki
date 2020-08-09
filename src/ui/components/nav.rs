use crate::ui::style;
// use crate::ui::style;
// use crate::ui::components::*;
use crate::app::{App, Event, Message};
use iced::{
    button, image, widget::Container, Button, Command, Element, HorizontalAlignment, Length, Row,
    Text,
};

#[derive(Debug, Clone)]
pub struct CurrentMediaPress {
    selected: bool,
}

impl Event for CurrentMediaPress {
    fn handle(self, app: &mut App) -> Command<Message> {
        if !self.selected {
            println!("pressed media");
            app.nav.settings_selected = false;
            app.nav.anime_selected = false;
            app.nav.manga_selected = false;
            app.nav.media_selected = true;
            app.page.change_page(super::Page::CurrentMedia);
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct SettingsPress {
    selected: bool,
}

impl Event for SettingsPress {
    fn handle(self, app: &mut App) -> Command<Message> {
        if !self.selected {
            println!("pressed settings");
            app.nav.settings_selected = true;
            app.nav.anime_selected = false;
            app.nav.manga_selected = false;
            app.nav.media_selected = false;
            app.page.change_page(super::Page::Settings);
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct AnimeListPress {
    selected: bool,
}

impl Event for AnimeListPress {
    fn handle(self, app: &mut App) -> Command<Message> {
        if !self.selected {
            println!("pressed anime list");
            app.nav.settings_selected = false;
            app.nav.anime_selected = true;
            app.nav.manga_selected = false;
            app.nav.media_selected = false;
            app.page.change_page(super::Page::Anime);
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct MangaListPress {
    selected: bool,
}

impl Event for MangaListPress {
    fn handle(self, app: &mut App) -> Command<Message> {
        if !self.selected {
            println!("pressed manga list");
            app.nav.settings_selected = false;
            app.nav.anime_selected = false;
            app.nav.manga_selected = true;
            app.nav.media_selected = false;
            app.page.change_page(super::Page::Manga);
        }
        Command::none()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Nav {
    anime_state: button::State,
    manga_state: button::State,
    media_state: button::State,
    settings_state: button::State,
    refresh_state: button::State,
    media_selected: bool,
    settings_selected: bool,
    anime_selected: bool,
    manga_selected: bool,
    avatar: Option<image::Handle>,
    // content: Page,
}

impl Nav {
    pub fn new() -> Self {
        Nav {
            media_selected: true,
            ..Nav::default()
        }
    }

    pub fn update(&mut self, app: &mut App, message: Message) {
        message.handle(app);
    }

    fn nav_button<'a>(
        state: &'a mut button::State,
        label: &str,
        selected: bool,
        msg: Message,
    ) -> Element<'a, Message> {
        let text_size = 18;
        let padding_size = 16;
        Button::new(
            state,
            Text::new(label)
                .size(text_size)
                .horizontal_alignment(HorizontalAlignment::Center),
        )
        .padding(padding_size)
        .style(style::Button::Nav { selected })
        .on_press(msg)
        .into()
    }

    pub fn view(&mut self) -> Element<Message> {
        let anime = Self::nav_button(
            &mut self.anime_state,
            "Anime List",
            self.anime_selected,
            AnimeListPress {
                selected: self.anime_selected,
            }
            .into(),
        );
        let manga = Self::nav_button(
            &mut self.manga_state,
            "Manga List",
            self.manga_selected,
            MangaListPress {
                selected: self.manga_selected,
            }
            .into(),
        );
        let media = Self::nav_button(
            &mut self.media_state,
            "Current Media",
            self.media_selected,
            CurrentMediaPress {
                selected: self.media_selected,
            }
            .into(),
        );
        let settings = Self::nav_button(
            &mut self.settings_state,
            "Settings",
            self.settings_selected,
            SettingsPress {
                selected: self.settings_selected,
            }
            .into(),
        );

        let left_spacer = Container::new(Text::new("")).width(Length::FillPortion(4));
        let right_spacer = Container::new(Text::new("")).width(Length::FillPortion(5));

        let avatar = match &self.avatar {
            Some(avatar) => Some(image::Image::new(avatar.clone())),
            None => None,
        };

        let mut nav = Row::new().spacing(0);

        if let Some(avatar) = avatar {
            nav = nav.push(avatar.height(Length::Units(52)));
        }

        nav = nav
            .push(left_spacer)
            .push(anime)
            .push(manga)
            .push(media)
            .push(settings)
            .push(right_spacer);

        Container::new(nav)
            .style(style::Container::NavBackground)
            .into()
    }

    pub fn set_avatar(&mut self, avatar: Option<image::Handle>) {
        self.avatar = avatar;
    }
}
