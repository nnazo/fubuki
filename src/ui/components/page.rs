use iced::{Element, Length, Row, Text};

#[derive(Debug, Clone)]
pub enum Message {
    MediaFound(String),
    MediaNotFound,
}

#[derive(Clone, Debug)]
pub enum Page {
    // Loading, -- this should probably not be with these
    CurrentMedia { current: String },
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Page::CurrentMedia {
            current: "None Found".to_string(),
        }
    }
}

impl<'a> Page {
    pub fn update(&mut self, msg: Message) {
        match self {
            Self::CurrentMedia { current } => {
                match msg {
                    Message::MediaFound(title) => {
                        if title != current.clone() {
                            // title.clone_into(current);
                            current.clone_from(&title);
                        }
                    }
                    Message::MediaNotFound => {
                        current.clone_from(&"None Found".to_string());
                    }
                }
            }
            Self::Settings => {}
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self {
            Self::CurrentMedia { current } => Self::current_media(current.clone()).into(),
            Self::Settings => Self::settings().into(),
        }
    }

    fn container() -> Row<'a, Message> {
        Row::new()
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill)
    }

    fn current_media(current: String) -> Row<'a, Message> {
        Self::container().push(Text::new(current))
    }

    fn settings() -> Row<'a, Message> {
        Self::container()
    }
}
