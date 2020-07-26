use iced::{Element, Length, Row, Text, Column, Container};
use crate::{anilist, ui::style};

#[derive(Debug, Clone)]
pub enum Message {
    MediaFound(anilist::MediaList),
    MediaNotFound,
}

#[derive(Clone, Debug)]
pub enum Page {
    // Loading, -- this should probably not be with these
    CurrentMedia { current: Option<anilist::MediaList> },
    Settings,
}

impl Page {
    fn replace_current_media(&mut self, media_list: Option<anilist::MediaList>) {
        match self {
            Page::CurrentMedia { current } => {
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
}

impl Default for Page {
    fn default() -> Self {
        Page::CurrentMedia {
            current: None,
        }
    }
}

impl<'a> Page {
    pub fn update(&mut self, msg: Message) {
        match self {
            Self::CurrentMedia { current: _ } => {
                match msg {
                    Message::MediaFound(media_list) => {
                        self.replace_current_media(Some(media_list));
                    }
                    Message::MediaNotFound => {
                        self.replace_current_media(None);
                    }
                }
            }
            Self::Settings => {}
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self {
            Self::CurrentMedia { current } => Self::current_media(current).into(),
            Self::Settings => Self::settings().into(),
        }
    }

    fn container(content: Element<'a, Message>) -> Container<'a, Message> {
        Container::new(Row::new().push(content))
            .height(Length::Fill)
            .width(Length::Fill)
            .style(style::Container::Background)
    }

    fn current_media(current: &Option<anilist::MediaList>) -> Container<'a, Message> {
        let mut cols = Column::<Message>::new().spacing(15);
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
                    Some(title) => cols = cols.push(Text::new(title)),
                    None => cols = cols.push(Text::new("Could Not Get Title")),
                }
            },
            None => {
                cols = cols.push(Text::new("No Media Detected"));
            },
        }
        Self::container(cols.into())
    }

    fn settings() -> Container<'a, Message> {
        Self::container(Text::new("").into())
    }
}
