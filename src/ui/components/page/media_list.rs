use crate::{
    anilist,
    app::{App, Event, Message},
    ui::style,
};
use iced::{
    button, scrollable, Button, Column, Command, Container, Element, Length, Row, Scrollable, Text,
};
use once_cell::sync::Lazy;
use std::default::Default;

#[derive(Debug, Default, Clone)]
pub struct MediaListPage {
    list: Option<anilist::MediaListCollection>,
    media_type: anilist::MediaType,
    selected_index: usize,
    list_selection_btn_states: Vec<button::State>,
    list_scroll_state: scrollable::State,
}

impl MediaListPage {
    pub fn new(media_type: anilist::MediaType) -> Self {
        Self {
            media_type,
            ..Self::default()
        }
    }

    pub fn set_list(&mut self, list: Option<anilist::MediaListCollection>) {
        self.list = list;
        match &mut self.list {
            Some(list) => {
                if let Some(lists) = &mut list.lists {
                    self.list_selection_btn_states.clear();
                    self.list_selection_btn_states
                        .resize(lists.len(), button::State::default());
                }
            }
            None => {
                self.list_selection_btn_states.clear();
            }
        }
    }

    pub fn get_list_mut(&mut self) -> Option<&mut anilist::MediaListCollection> {
        self.list.as_mut()
    }

    pub fn get_list(&self) -> Option<&anilist::MediaListCollection> {
        self.list.as_ref()
    }

    pub fn change_list_group(&mut self, index: usize) {
        self.selected_index = index;
    }

    // pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        let media_type = self.media_type.clone();
        let button_states = &mut self.list_selection_btn_states;
        let list = self.list.as_mut();
        match list {
            Some(list) => {
                let mut row = Row::new().spacing(12).push(Self::container(
                    Self::list_selection_view(
                        button_states,
                        list,
                        media_type.clone(),
                        self.selected_index,
                    )
                    .into(),
                ));

                if let Some(list_view) = Self::list_view(
                    &mut self.list_scroll_state,
                    list,
                    media_type.clone(),
                    self.selected_index,
                ) {
                    row = row.push(Self::container(list_view));
                }

                Self::container(row.into())
            }
            None => {
                let message = format!(
                    "It seems you have not tracked any {}, or you are not logged in.",
                    self.media_type.string().to_lowercase()
                );
                Self::container(Text::new(message).into())
            }
        }
    }

    pub fn list_selection_view<'a>(
        button_states: &'a mut Vec<button::State>,
        list: &mut anilist::MediaListCollection,
        media_type: anilist::MediaType,
        selected_index: usize,
    ) -> Element<'a, Message> {
        let mut list_buttons = Column::new().spacing(2);
        if let Some(lists) = &mut list.lists {
            let mut i = 0usize;
            let group_names: Vec<&String> = lists
                .iter()
                .filter_map(|media_list_group| media_list_group.as_ref())
                .filter_map(|media_list_group| media_list_group.name.as_ref())
                .collect();
            let mut button_states = button_states.iter_mut();
            for name in group_names {
                if let Some(button_state) = button_states.next() {
                    let button = Self::list_selection_button(
                        button_state,
                        name,
                        i,
                        selected_index,
                        media_type.clone(),
                    );
                    i += 1;
                    if let Some(button) = button {
                        list_buttons = list_buttons.push(button);
                    }
                }
            }
        }
        list_buttons.into()
    }

    pub fn list_selection_button<'a>(
        button_state: &'a mut button::State,
        name: &str,
        index: usize,
        selected_index: usize,
        media_type: anilist::MediaType,
    ) -> Option<Element<'a, Message>> {
        let text_size = 16;
        Some(
            Button::new(button_state, Text::new(name).size(text_size))
                .width(Length::Units(128))
                .on_press(ListGroupSelected { index, media_type }.into())
                .style(style::Button::ListGroup {
                    selected: index == selected_index,
                })
                .into(),
        )
    }

    pub fn list_view<'a>(
        scroll_state: &'a mut scrollable::State,
        list: &'a mut anilist::MediaListCollection,
        media_type: anilist::MediaType,
        index: usize,
    ) -> Option<Element<'a, Message>> {
        let scroll = Scrollable::new(scroll_state);
        let header = Self::header_row(media_type);
        let mut col = Column::new().spacing(4);
        if let Some(header) = header {
            col = col.push(header);
        }

        let group = list.lists.as_mut()?.get_mut(index)?.as_mut()?;
        let entries: Vec<&mut anilist::MediaList> = group
            .entries
            .as_mut()?
            .iter_mut()
            .filter_map(|entry| entry.as_mut())
            .collect();
        for entry in entries {
            if let Some(entry_row) = Self::entry_row(entry) {
                col = col.push(entry_row);
            }
        }

        Some(scroll.push(col).into())
    }

    pub fn header_row(media_type: anilist::MediaType) -> Option<Element<'static, Message>> {
        let text_size = 14;
        let mut row = Row::new().width(Length::Fill).spacing(8);
        static ANIME_LABELS: Lazy<Vec<&str>> =
            Lazy::new(|| vec!["Title", "Score", "Progress", "Format"]);
        static MANGA_LABELS: Lazy<Vec<&str>> =
            Lazy::new(|| vec!["Title", "Score", "Chapters", "Volumes", "Format"]);
        let labels = match media_type {
            anilist::MediaType::Anime => &ANIME_LABELS,
            anilist::MediaType::Manga => &MANGA_LABELS,
        };
        static ANIME_PORTIONS: Lazy<Vec<u16>> = Lazy::new(|| vec![3, 1, 1, 1]);
        static MANGA_PORTIONS: Lazy<Vec<u16>> = Lazy::new(|| vec![3, 1, 1, 1, 1]);
        let mut fill_portions = match labels.len() {
            4 => Some(&ANIME_PORTIONS),
            5 => Some(&MANGA_PORTIONS),
            _ => None,
        }?
        .iter();
        for label in labels.iter() {
            let length = fill_portions.next();
            row = row.push(
                Text::new(*label)
                    .size(text_size)
                    .width(Length::FillPortion(*length.unwrap_or(&1u16))),
            );
        }
        Some(Self::entry_container(row))
    }

    pub fn entry_row(entry: &mut anilist::MediaList) -> Option<Element<Message>> {
        let media = entry.media.as_ref()?;
        let text_size = 12;

        static ANIME_PORTIONS: Lazy<Vec<u16>> = Lazy::new(|| vec![3, 1, 1, 1]);
        static MANGA_PORTIONS: Lazy<Vec<u16>> = Lazy::new(|| vec![3, 1, 1, 1, 1]);
        let mut fill_portions = match media.media_type {
            Some(anilist::MediaType::Anime) => Some(&ANIME_PORTIONS),
            Some(anilist::MediaType::Manga) => Some(&MANGA_PORTIONS),
            _ => None,
        }?
        .iter();

        let title = {
            let title = match media.preferred_title() {
                Some(preferred) => preferred,
                None => media.title.as_ref()?.romaji.as_ref()?.clone(),
            };
            let fill = fill_portions.next().unwrap_or(&1u16);
            Text::new(title)
                .size(text_size)
                .width(Length::FillPortion(*fill))
        };
        let mut fill = fill_portions.next().unwrap_or(&1u16);
        let score = Text::new(format!("{}", entry.score.unwrap_or_default()))
            .size(text_size)
            .width(Length::FillPortion(*fill));
        fill = fill_portions.next().unwrap_or(&1u16);
        let progress = Text::new(entry.progress_string())
            .size(text_size)
            .width(Length::FillPortion(*fill));

        let mut row = Row::new()
            .width(Length::Fill)
            .spacing(8)
            .push(title)
            .push(score)
            .push(progress);

        match media.media_type {
            Some(anilist::MediaType::Manga) => {
                fill = fill_portions.next().unwrap_or(&1u16);
                let progress_vol = Text::new(entry.progress_volumes_string())
                    .size(text_size)
                    .width(Length::FillPortion(*fill));
                row = row.push(progress_vol);
            }
            _ => {}
        }

        let format = {
            let format = match &media.format {
                Some(format) => format.str(),
                None => "Unknown",
            };
            fill = fill_portions.next().unwrap_or(&1u16);
            Text::new(format)
                .size(text_size)
                .width(Length::FillPortion(*fill))
        };

        Some(Self::entry_container(row.push(format)))
    }

    pub fn container(element: Element<Message>) -> Element<Message> {
        Container::new(element).padding(12).into()
    }

    pub fn entry_container(row: Row<Message>) -> Element<Message> {
        Container::new(row)
            .padding(12)
            .style(style::Container::EntryRow)
            .into()
    }
}

#[derive(Debug, Clone)]
pub struct ListGroupSelected {
    index: usize,
    media_type: anilist::MediaType,
}

impl Event for ListGroupSelected {
    fn handle(self, app: &mut App) -> Command<Message> {
        let list_page = match self.media_type {
            anilist::MediaType::Anime => &mut app.page.anime,
            anilist::MediaType::Manga => &mut app.page.manga,
        };
        list_page.change_list_group(self.index);
        Command::none()
    }
}
