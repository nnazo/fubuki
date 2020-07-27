use iced::{Element, Length, Row, Text, Column, Container, image};
use crate::{anilist, ui::style};

#[derive(Debug, Clone)]
pub enum Message {
    MediaFound(anilist::MediaList),
    MediaNotFound,
    CoverChange(Option<image::Handle>),
}

#[derive(Clone, Debug)]
pub enum Page {
    // Loading, -- this should probably not be with these
    CurrentMedia {
        current: Option<anilist::MediaList>,
        cover: Option<image::Handle>,
    },
    Settings,
}

impl Page {
    fn replace_current_media(&mut self, media_list: Option<anilist::MediaList>) {
        match self {
            Page::CurrentMedia { current, cover: _ } => {
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
            Page::CurrentMedia { current: _, cover } => {
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
        }
    }
}

impl<'a> Page {
    pub fn update(&mut self, msg: Message) {
        match self {
            Self::CurrentMedia { current: _, cover: _ } => {
                match msg {
                    Message::MediaFound(media_list) => {
                        self.replace_current_media(Some(media_list));
                    }
                    Message::MediaNotFound => {
                        self.replace_current_media(None);
                        self.replace_media_cover(None);
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
                    }
                }
            }
            Self::Settings => {}
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self {
            Self::CurrentMedia { current, cover } => Self::current_media(current, cover).into(),
            Self::Settings => Self::settings().into(),
        }
    }

    fn container(content: Element<'a, Message>) -> Container<'a, Message> {
        Container::new(Row::new().push(content))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(style::Container::Background)
    }

    fn current_media(current: &mut Option<anilist::MediaList>, cover: &Option<image::Handle>) -> Container<'a, Message> {
        let padding_size = 24;
        let spacing_size = 12;
        let inner_col_space = 6;
        let mut row = Row::<Message>::new().padding(padding_size).spacing(padding_size);
        if let Some(cover) = cover {
            row = row.push(image::Image::new(cover.clone()));
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

    fn settings() -> Container<'a, Message> {
        Self::container(Text::new("").into())
    }
}
