use crate::ui::style;
// use crate::ui::style;
// use crate::ui::components::*;
use iced::{button, Button, Element, Length, Row, Text, image, widget::Container, HorizontalAlignment, Command};
use crate::app::{Event, Message, App};

// AnimeListPress,
// MangaListPress,

#[derive(Debug, Clone)]
pub struct CurrentMediaPress {
    selected: bool,
}

impl Event for CurrentMediaPress {
    fn handle(self, app: &mut App) -> Command<Message> {
        if !self.selected {
            println!("pressed media");
            app.nav.settings_selected = false;
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
            app.nav.media_selected = false;
            app.page.change_page(super::Page::Settings);
        }
        Command::none()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Nav {
    // anime_state: button::State,
    // manga_state: button::State,
    media_state: button::State,
    settings_state: button::State,
    refresh_state: button::State,
    media_selected: bool,
    settings_selected: bool,
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

    pub fn view(&mut self) -> Element<Message> {
        let text_size = 18;
        let padding_size = 16;
        let media = Button::new(
                &mut self.media_state, 
                Text::new("Current Media")
                    .size(text_size)
                    .horizontal_alignment(HorizontalAlignment::Center)
            )
            .padding(padding_size)
            .style(style::Button::Nav {
                selected: self.media_selected,
            })
            .on_press(CurrentMediaPress {
                selected: self.media_selected,
            }.into());

        let settings = Button::new(
                &mut self.settings_state, 
                Text::new("Settings")
                    .size(text_size)
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
            .padding(padding_size)
            .style(style::Button::Nav {
                selected: self.settings_selected,
            })
            .on_press(SettingsPress {
                selected: self.settings_selected,
            }.into());

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
            .push(media)
            .push(settings)
            .push(right_spacer);
        
        Container::new(nav)
            .style(style::Container::NavBackground)
            .into()
    }

    pub fn set_avatar(&mut self, avatar: image::Handle) {
        self.avatar = Some(avatar);
    }
}
