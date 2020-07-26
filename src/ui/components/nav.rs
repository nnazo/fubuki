use crate::ui::style;
// use crate::ui::style;
// use crate::ui::components::*;
use iced::{button, Button, Element, Length, Row, Text, image, widget::Container, HorizontalAlignment};

#[derive(Debug, Clone)]
pub enum Message {
    // AnimeListPress,
    // MangaListPress,
    CurrentMediaPress { selected: bool },
    SettingsPress { selected: bool },
    RefreshLists,
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

    pub fn update(&mut self, message: Message) {
        match message {
            Message::CurrentMediaPress { selected: _ } => {
                self.media_selected = true;
                self.settings_selected = false;
            }
            Message::SettingsPress { selected: _ } => {
                self.media_selected = false;
                self.settings_selected = true;
            },
            Message::RefreshLists => {
                println!("list refresh pressed");
            },
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let media = Button::new(
                &mut self.media_state, 
                Text::new("Current Media")
                    .horizontal_alignment(HorizontalAlignment::Center)
            )
            .padding(16)
            .width(Length::Fill)
            .style(style::Button::Nav {
                selected: self.media_selected,
            })
            .on_press(Message::CurrentMediaPress {
                selected: self.media_selected,
            });

        let settings = Button::new(
                &mut self.settings_state, 
                Text::new("Settings")
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
            .padding(16)
            .width(Length::Fill)
            .style(style::Button::Nav {
                selected: self.settings_selected,
            })
            .on_press(Message::SettingsPress {
                selected: self.settings_selected,
            });

        let left_spacer = Container::new(Text::new("")).width(Length::FillPortion(2));
        let right_spacer = Container::new(Text::new("")).width(Length::Fill);    

        let refresh = Button::new(&mut self.refresh_state, Text::new("Refresh"))
            .padding(16)
            .style(style::Button::Nav {
                selected: false,
            })
            .on_press(Message::RefreshLists);

        let avatar = match &self.avatar {
            Some(avatar) => Some(image::Image::new(avatar.clone())),
            None => None,
        };

        let mut nav = Row::new().spacing(0);

        if let Some(avatar) = avatar {
            nav = nav.push(avatar.height(Length::Units(52)));
        }

        nav = nav
            .push(refresh)
            .push(right_spacer)
            .push(media)
            .push(settings)
            .push(left_spacer);
        
        Container::new(nav)
            .style(style::Container::NavBackground)
            .into()
    }

    pub fn set_avatar(&mut self, avatar: image::Handle) {
        self.avatar = Some(avatar);
    }
}
