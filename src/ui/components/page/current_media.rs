use iced::{Element, Row, Text, Column, image, Command};
use crate::{anilist, recognition, app::{App, Event, Message}};
use super::PageContainer;

#[derive(Debug, Clone)]
pub struct CurrentMediaPage {
    current: Option<(anilist::MediaList, recognition::Media)>,
    cover: Option<image::Handle>,
    default_cover: image::Handle,
}

impl CurrentMediaPage {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        let padding_size = 24;
        let spacing_size = 12;
        let inner_col_space = 6;
        let mut row = Row::<Message>::new().padding(padding_size).spacing(padding_size);
        match &mut self.cover {
            Some(cover) => row = row.push(image::Image::new(cover.clone())),
            None => row = row.push(image::Image::new(self.default_cover.clone())),
        }
        let mut col = Column::<Message>::new().spacing(spacing_size);
        let title_size = 18;
        let text_size = 14;
        match &mut self.current {
            Some((current, current_detected)) => {
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
                // current.current_media_string();
                col = col.push(Text::new(current_detected.current_media_string()).size(text_size));
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
        PageContainer::container(row.push(col).into()).into()
    }
    
    pub fn set_current_media(&mut self, media_list: Option<(anilist::MediaList, recognition::Media)>) {
        self.current = media_list;
    }

    pub fn set_media_cover(&mut self, new_cover: Option<image::Handle>) {
        // self.cover.clone_from(&new_cover); 
        self.cover = new_cover;
    }
}

impl Default for CurrentMediaPage {
    fn default() -> Self {
        CurrentMediaPage {
            current: None,
            cover: None,
            default_cover: image::Handle::from("./res/cover_default.jpg"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoverChange(pub Option<image::Handle>);

impl Event for CoverChange {
    fn handle(self, app: &mut App) -> Command<Message> {
        let CoverChange(cover) = self;
        app.page.current_media.set_media_cover(cover);
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct MediaChange(pub Option<(anilist::MediaList, recognition::Media)>);

impl Event for MediaChange {
    fn handle(self, app: &mut App) -> Command<Message> {
        let MediaChange(media_list) = self;
        match media_list {
            Some(media) => {
                app.page.current_media.set_current_media(Some(media));
            }
            None => {
                app.page.current_media.set_current_media(None);
                app.page.current_media.set_media_cover(None);
            }
        }
        Command::none()
    }
}