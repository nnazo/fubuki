use super::PageContainer;
use crate::{
    anilist,
    app::{App, Event, Message},
    recognition,
    ui::style,
};
use iced::{
    button, image, Button, Column, Command, Element, HorizontalAlignment, Length, Row, Text,
};

#[derive(Debug, Clone)]
pub struct CurrentMediaPage {
    update_cancel_btn_state: button::State,
    show_cancel_update: bool,
    current: Option<anilist::MediaList>,
    recognized: Option<recognition::Media>,
    cover: Option<image::Handle>,
    default_cover: image::Handle,
}

impl CurrentMediaPage {
    pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        let spacing_size = 12;
        let inner_col_space = 6;
        let button_padding = 12;
        let mut row = Row::<Message>::new().spacing(24);
        match &mut self.cover {
            Some(cover) => row = row.push(image::Image::new(cover.clone())),
            None => row = row.push(image::Image::new(self.default_cover.clone())),
        }
        let mut col = Column::<Message>::new().spacing(spacing_size);
        let title_size = 18;
        let text_size = 14;
        match &mut self.current {
            Some(current) => {
                let mut inner_row = Row::<Message>::new();
                let title = match &current.media {
                    Some(media) => match media.preferred_title() {
                        Some(title) => Some(title.clone()),
                        None => None,
                    },
                    None => None,
                };
                match title {
                    Some(title) => inner_row = inner_row.push(Text::new(title).size(title_size)),
                    None => {
                        inner_row =
                            inner_row.push(Text::new("Could Not Get Title").size(title_size))
                    }
                }
                if self.show_cancel_update {
                    inner_row = inner_row.push(Text::new("").width(Length::Fill)).push(
                        Button::new(
                            &mut self.update_cancel_btn_state,
                            Text::new("Cancel")
                                .size(text_size)
                                .horizontal_alignment(HorizontalAlignment::Center),
                        )
                        .padding(button_padding)
                        .style(style::Button::Danger)
                        .on_press(CancelListUpdate(current.media_id, false).into()),
                    );
                }
                col = col.push(inner_row);
                // current.current_media_string();
                if let Some(current_detected) = &self.recognized {
                    col = col
                        .push(Text::new(current_detected.current_media_string()).size(text_size));
                    if let Some(media) = &mut current.media {
                        if let Some(desc) = media.description() {
                            col = col.push(
                                Column::new()
                                    .spacing(inner_col_space)
                                    .push(Text::new("Description:").size(text_size))
                                    .push(Text::new(desc.clone()).size(text_size)),
                            );
                        }
                    }
                }
            }
            None => {
                row = row.push(Text::new("No Media Detected").size(title_size));
            }
        }
        PageContainer::container(row.push(col).into()).into()
    }

    pub fn show_cancel_button(&mut self, show: bool) {
        self.show_cancel_update = show;
    }

    pub fn set_current_media(
        &mut self,
        media_list: Option<anilist::MediaList>,
        recognized: Option<recognition::Media>,
    ) {
        self.current = media_list;
        self.recognized = recognized;
    }

    pub fn set_media_cover(&mut self, new_cover: Option<image::Handle>) {
        // self.cover.clone_from(&new_cover);
        self.cover = new_cover;
    }
}

impl Default for CurrentMediaPage {
    fn default() -> Self {
        CurrentMediaPage {
            update_cancel_btn_state: button::State::default(),
            show_cancel_update: false,
            current: None,
            recognized: None,
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
pub struct MediaChange(
    pub Option<anilist::MediaList>,
    pub Option<recognition::Media>,
    pub bool,
);

impl Event for MediaChange {
    fn handle(self, app: &mut App) -> Command<Message> {
        let MediaChange(media_list, recognized, needs_update) = self;
        println!("setting cancel button {}", needs_update);
        app.page.current_media.show_cancel_button(needs_update);
        if media_list.is_none() {
            app.page.current_media.set_media_cover(None);
        }
        app.page
            .current_media
            .set_current_media(media_list, recognized);
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct CancelListUpdate(pub i32, pub bool);

impl Event for CancelListUpdate {
    fn handle(self, app: &mut App) -> Command<Message> {
        let CancelListUpdate(media_id, already_sent) = self;
        app.page.current_media.show_cancel_button(false);
        if !already_sent {
            let index = app.updates.find_index(media_id);
            match index {
                Some(index) => match app.updates.remove(index) {
                    Some(_) => println!("successfully removed media_id {} from queue", media_id),
                    None => println!("removal returned None for media_id {} in queue", media_id),
                },
                None => eprintln!("could not find media_id {} in list update queue", media_id),
            }
        }
        Command::none()
    }
}
