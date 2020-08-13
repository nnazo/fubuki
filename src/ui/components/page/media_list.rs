use crate::{
    anilist,
    app::{App, Event, Message},
    ui::style,
};
use iced::{
    button, scrollable, text_input, Align, Button, Column, Command, Container, Element, Length,
    Row, Scrollable, Text, TextInput, VerticalAlignment,
};
use once_cell::sync::Lazy;
use std::default::Default;

#[derive(Debug, Default, Clone)]
pub struct MediaListPage {
    list: Option<anilist::MediaListCollection>,
    media_type: anilist::MediaType,
    selected_index: usize,
    filter: String,
    filter_state: text_input::State,
    list_selection_btn_states: Vec<button::State>,
    list_scroll_state: scrollable::State,
    inc_progress_btn_states: Vec<button::State>,
    inc_progress_vol_btn_states: Vec<button::State>,
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
        match &self.list {
            Some(list) => {
                if let Some(lists) = &list.lists {
                    self.list_selection_btn_states.clear();
                    self.list_selection_btn_states
                        .resize(lists.len(), button::State::default());

                    let entry_count = list.count_entries();
                    match self.media_type {
                        anilist::MediaType::Anime => {
                            self.inc_progress_btn_states.clear();
                            self.inc_progress_btn_states
                                .resize(entry_count, button::State::default());
                        }
                        anilist::MediaType::Manga => {
                            self.inc_progress_btn_states.clear();
                            self.inc_progress_btn_states
                                .resize(entry_count, button::State::default());
                            self.inc_progress_vol_btn_states.clear();
                            self.inc_progress_vol_btn_states
                                .resize(entry_count, button::State::default());
                        }
                    }
                }
            }
            None => {
                self.list_selection_btn_states.clear();
                self.inc_progress_btn_states.clear();
                self.inc_progress_vol_btn_states.clear();
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

    pub fn set_filter(&mut self, value: String) {
        self.filter = value;
    }

    // pub fn update(&mut self, _msg: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        let media_type = *&self.media_type;
        let button_states = &mut self.list_selection_btn_states;
        let list = self.list.as_ref();
        match list {
            Some(list) => {
                let mut row = Row::new().spacing(12).push(Self::container(
                    Column::new()
                        .spacing(12)
                        .push(Self::list_selection_view(
                            button_states,
                            list,
                            media_type.clone(),
                            self.selected_index,
                        ))
                        .push(Self::filter(
                            &mut self.filter_state,
                            &self.filter,
                            *&self.media_type,
                        ))
                        .into(),
                ));

                if let Some(list_view) = Self::list_view(
                    &mut self.list_scroll_state,
                    list,
                    &self.filter,
                    *&media_type,
                    self.selected_index,
                    &mut self.inc_progress_btn_states,
                    &mut self.inc_progress_vol_btn_states,
                ) {
                    row = row.push(Self::container(list_view));
                }

                Self::container(row.into())
            }
            None => {
                let message = format!(
                    "Oh no! It seems like you have not tracked any {}.",
                    self.media_type.string().to_lowercase()
                );
                Self::container(Text::new(message).size(18).into())
            }
        }
    }

    pub fn list_selection_view<'a>(
        button_states: &'a mut Vec<button::State>,
        list: &anilist::MediaListCollection,
        media_type: anilist::MediaType,
        selected_index: usize,
    ) -> Element<'a, Message> {
        let mut list_buttons = Column::new().spacing(2);
        if let Some(lists) = &list.lists {
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
            Button::new(
                button_state,
                Text::new(name)
                    .size(text_size)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .width(Length::Units(128))
            .height(Length::Units(32))
            .on_press(ListGroupSelected { index, media_type }.into())
            .style(style::Button::ListGroup {
                selected: index == selected_index,
            })
            .into(),
        )
    }

    pub fn list_view<'a>(
        scroll_state: &'a mut scrollable::State,
        list: &'a anilist::MediaListCollection,
        filter: &str,
        media_type: anilist::MediaType,
        index: usize,
        inc_progress_btn_states: &'a mut Vec<button::State>,
        inc_progress_vol_btn_states: &'a mut Vec<button::State>,
    ) -> Option<Element<'a, Message>> {
        let scroll = Scrollable::new(scroll_state);
        let header = Self::header_row(media_type);
        let mut col = Column::new().spacing(4);
        if let Some(header) = header {
            col = col.push(header);
        }

        let group = list.lists.as_ref()?.get(index)?.as_ref()?;
        let entries: Vec<&anilist::MediaList> = group
            .entries
            .as_ref()?
            .iter()
            .filter_map(|entry| entry.as_ref())
            .filter(|entry| entry.titles_contain(filter))
            .collect();
        let mut inc_button_state = inc_progress_btn_states.iter_mut();
        let mut inc_vol_button_state = inc_progress_vol_btn_states.iter_mut();
        for entry in entries {
            if let Some(entry_row) =
                Self::entry_row(entry, inc_button_state.next(), inc_vol_button_state.next())
            {
                col = col.push(entry_row);
            }
        }

        Some(scroll.push(col).into())
    }

    pub fn filter<'a>(
        state: &'a mut text_input::State,
        value: &str,
        media_type: anilist::MediaType,
    ) -> Element<'a, Message> {
        let text_size = 16;
        TextInput::new(state, "Filter list", value, move |value| {
            ListFilterTextChange {
                value,
                media_type: *&media_type,
            }
            .into()
        })
        .size(text_size)
        .padding(6)
        .width(Length::Units(128))
        .style(style::Input)
        .into()
    }

    pub fn header_row(media_type: anilist::MediaType) -> Option<Element<'static, Message>> {
        let text_size = 14;
        let mut row = Row::new()
            .width(Length::Fill)
            .spacing(8)
            .align_items(Align::Center);
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

    pub fn entry_row<'a>(
        entry: &anilist::MediaList,
        inc_button_state: Option<&'a mut button::State>,
        inc_vol_button_state: Option<&'a mut button::State>,
    ) -> Option<Element<'a, Message>> {
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

        let progress = {
            Row::new()
                .width(Length::FillPortion(*fill))
                .spacing(4)
                .align_items(Align::Center)
                .push(Text::new(entry.progress_string()).size(text_size))
                .push(
                    Button::new(inc_button_state?, Text::new("+").size(10))
                        .style(style::Button::Increment)
                        .on_press(
                            IncrementMediaProgress {
                                media_id: media.id,
                                media_type: *media.media_type.as_ref()?,
                                is_volume_progress: false,
                            }
                            .into(),
                        ),
                )
        };

        let mut row = Row::new()
            .width(Length::Fill)
            .spacing(8)
            .align_items(Align::Center)
            .push(title)
            .push(score)
            .push(progress);

        match media.media_type {
            Some(anilist::MediaType::Manga) => {
                fill = fill_portions.next().unwrap_or(&1u16);
                let progress_vol = Row::new()
                    .width(Length::FillPortion(*fill))
                    .spacing(4)
                    .align_items(Align::Center)
                    .push(Text::new(entry.progress_volumes_string()).size(text_size))
                    .push(
                        Button::new(inc_vol_button_state?, Text::new("+").size(10))
                            .style(style::Button::Increment)
                            .on_press(
                                IncrementMediaProgress {
                                    media_id: media.id,
                                    media_type: *media.media_type.as_ref()?,
                                    is_volume_progress: true,
                                }
                                .into(),
                            ),
                    );
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
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let list_page = match self.media_type {
            anilist::MediaType::Anime => &mut app.page.anime,
            anilist::MediaType::Manga => &mut app.page.manga,
        };
        list_page.change_list_group(self.index);
        None
    }
}

#[derive(Debug, Clone)]
pub struct IncrementMediaProgress {
    media_id: i32,
    media_type: anilist::MediaType,
    is_volume_progress: bool,
}

impl Event for IncrementMediaProgress {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let list = match self.media_type {
            anilist::MediaType::Anime => app.page.anime.get_list_mut(),
            anilist::MediaType::Manga => app.page.manga.get_list_mut(),
        };

        let entry = list?.find_entry_by_id_mut(self.media_id)?;

        let (progress, cap) = match self.is_volume_progress {
            true => (
                &mut entry.progress_volumes,
                match &entry.media {
                    Some(media) => media.volumes,
                    None => None,
                },
            ),
            false => (
                &mut entry.progress,
                match self.media_type {
                    anilist::MediaType::Anime => match &entry.media {
                        Some(media) => media.episodes,
                        None => None,
                    },
                    anilist::MediaType::Manga => match &entry.media {
                        Some(media) => media.chapters,
                        None => None,
                    },
                },
            ),
        };

        let progress = progress.as_mut()?;
        match cap {
            Some(cap) => {
                if *progress < cap {
                    *progress += 1;
                    app.updates.enqueue(entry.clone());
                }
            }
            None => {
                *progress += 1;
                app.updates.enqueue(entry.clone());
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct ListFilterTextChange {
    value: String,
    media_type: anilist::MediaType,
}

impl Event for ListFilterTextChange {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        match self.media_type {
            anilist::MediaType::Anime => app.page.anime.set_filter(self.value),
            anilist::MediaType::Manga => app.page.manga.set_filter(self.value),
        }
        None
    }
}
