use iced::{Element, Length, Row, Text, Column, Container, image, button, Button, HorizontalAlignment, Command};
use crate::{anilist, ui::style};

// Note:
// The reason I'm using a separate message enum here instead of the Message with the
// and using the Event trait in the app module is that I need a way of accessing the
// Page variant itself and the Event trait doesn't let me do that, so I map this
// message to a Message in app called PageMessage

#[derive(Debug, Clone)]
pub enum Message {
    MediaChange(Option<anilist::MediaList>),
    CoverChange(Option<image::Handle>),
    RefreshLists,
}

#[derive(Clone, Debug)]
pub enum Page {
    // Loading, -- this should probably not be with these
    CurrentMedia {
        current: Option<anilist::MediaList>,
        cover: Option<image::Handle>,
        default_cover: image::Handle,
    },
    Settings {
        refresh_list_state: button::State,
    },
}

impl Page {
    fn replace_current_media(&mut self, media_list: Option<anilist::MediaList>) {
        match self {
            Page::CurrentMedia { current, cover: _, default_cover: _ } => {
                match media_list {
                    Some(media_list) => match current {
                        Some(curr) => {
                            let pref_title = |media_list: &anilist::MediaList| match &media_list.media {
                                Some(media) => media.preferred_title(),
                                None => None,
                            };
                            let same_title = |title, curr_title| title == curr_title;
                            match pref_title(&media_list) {
                                Some(title) => {
                                    match pref_title(curr) {
                                        Some(curr_title) => {
                                            if !same_title(title, curr_title) {
                                                current.replace(media_list);
                                            }
                                        },
                                        None => {},
                                    }
                                },
                                None => {},
                            }
                        }
                        None => {
                            current.replace(media_list);
                        },
                    },
                    None => current.clone_from(&None::<anilist::MediaList>),
                }
                
            }
            _ => {},
        }
    }
        
    fn replace_media_cover(&mut self, new_cover: Option<image::Handle>) {
        match self {
            Page::CurrentMedia { current: _, cover, default_cover: _ } => {
                cover.clone_from(&new_cover);
            },
            _ => {},
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page::CurrentMedia {
            current: None,
            cover: None,
            default_cover: image::Handle::from("./res/cover_default.jpg"),
        }
    }
}

impl<'a> Page {
    pub fn update(&mut self, msg: Message) {
        match self {
            Self::CurrentMedia { current: _, cover: _, default_cover: _ } => {
                match msg {
                    Message::MediaChange(media_list) => {
                        match media_list {
                            Some(media_list) => {
                                self.replace_current_media(Some(media_list));
                            }
                            None => {
                                self.replace_current_media(None);
                                self.replace_media_cover(None);
                            }
                        }
                    }
                    Message::CoverChange(cover) => {
                        match cover {
                            Some(cover) => {
                                self.replace_media_cover(Some(cover));
                            },
                            None => {
                                self.replace_media_cover(None);
                            },
                        }
                    },
                    Message::RefreshLists => {},
                }
            }
            Self::Settings { refresh_list_state: _ } => {}
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self {
            Self::CurrentMedia { current, cover, default_cover } => Self::current_media(current, cover, default_cover).into(),
            Self::Settings { refresh_list_state: _ } => self.settings().into(),
        }
    }

    fn container(content: Element<'a, Message>) -> Container<'a, Message> {
        Container::new(Row::new().push(content))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(style::Container::Background)
    }

    fn current_media(current: &mut Option<anilist::MediaList>, cover: &Option<image::Handle>, default_cover: &image::Handle) -> Container<'a, Message> {
        let padding_size = 24;
        let spacing_size = 12;
        let inner_col_space = 6;
        let mut row = Row::<Message>::new().padding(padding_size).spacing(padding_size);
        match cover {
            Some(cover) => row = row.push(image::Image::new(cover.clone())),
            None => row = row.push(image::Image::new(default_cover.clone())),
        }
        let mut col = Column::<Message>::new().spacing(spacing_size);
        let title_size = 18;
        let text_size = 14;
        match current {
            Some(current) => {
                let title = match &current.media {
                    Some(media) => match media.preferred_title() {
                        Some(title) => Some(title.clone()),
                        None => None,
                    },
                    None => None,
                };
                match title {
                    Some(title) => col = col.push(Text::new(title).size(title_size)),
                    None => col = col.push(Text::new("Could Not Get Title").size(title_size)),
                }
                col = col.push(Text::new(current.current_media_string()).size(text_size));
                if let Some(media) = &mut current.media {
                    if let Some(desc) = media.description() {
                        col = col.push(
                            Column::new()
                                .spacing(inner_col_space)
                                .push(Text::new("Description:").size(text_size))
                                .push(Text::new(desc.clone()).size(text_size))
                        );    
                    }
                }
            },
            None => {
                row = row.push(Text::new("No Media Detected").size(title_size));
            },
        }
        Self::container(row.push(col).into())
    }

    fn settings(&'a mut self) -> Container<'a, Message> {
        let mut col = Column::new().padding(24).spacing(12);
        let text_size = 14;
        let button_padding = 12;

        match self {
            Page::Settings { refresh_list_state} => {
                col = col.push(Button::new(
                    refresh_list_state,
                    Text::new("Refresh Lists")
                        .size(text_size)
                        .horizontal_alignment(HorizontalAlignment::Center)
                )
                .padding(button_padding)
                .style(style::Button::Accent)
                .on_press(Message::RefreshLists)
                );
            },
            _ => {},
        }

        Self::container(col.into())
    }
}
