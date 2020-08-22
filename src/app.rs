use crate::{resources::Resources, *};
use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
use iced::{
    executor, time, window::Icon, Application, Column, Command, Container, Element, Length,
    Settings, Subscription,
};
use log::{debug, error, info, warn};
use recognition::MediaParser;
use std::fmt::Debug;
use ui::{components, style};

pub fn set_icon<T>(settings: &mut Settings<T>) -> Result<()> {
    match Resources::get("icon/fubuki.png") {
        Some(file) => {
            let image = image::load_from_memory(&*file)?;
            let rgba = image.to_rgba();
            let icon = Icon::from_rgba(rgba.to_vec(), rgba.width(), rgba.height())?;
            settings.window.icon = Some(icon);
            Ok(())
        }
        None => Err(anyhow!("could not get embedded res/icon/fubuki.png")),
    }
}

pub fn set_min_dimensions<T>(settings: &mut Settings<T>) {
    settings.window.min_size = Some((640, 480));
}

#[derive(Default)]
pub struct App {
    pub waiting_for_cover: bool,
    pub recognized: Option<recognition::Media>,
    pub media: Option<anilist::MediaList>,
    pub media_cover: Option<iced::image::Handle>,
    pub nav: components::Nav,
    pub page: components::PageContainer,
    pub user: Option<anilist::User>,
    pub updates: anilist::ListUpdateQueue,
}

impl App {
    fn query_user(token: String) -> Command<Message> {
        Command::perform(anilist::query_user(Some(token)), |result| match result {
            Ok(resp) => {
                if let Some(viewer_resp) = resp.data {
                    if let Some(user) = viewer_resp.viewer {
                        return UserFound(user).into();
                    }
                }
                NoMessage.into()
            }
            Err(err) => {
                error!("anilist user query failed: {}", err);
                NoMessage.into()
            }
        })
    }

    pub fn auth() -> Command<Message> {
        Command::perform(anilist::auth(), |result| match result {
            Ok(token) => Authorized(token).into(),
            Err(err) => {
                error!("authorization failed: {}", err);
                AuthFailed.into()
            }
        })
    }

    pub fn query_user_lists(token: String, user_id: i32) -> Command<Message> {
        Command::perform(
            anilist::query_media_lists(Some(token), user_id),
            |(anime_result, manga_result)| {
                let anime_list = match anime_result {
                    Ok(resp) => match resp.data {
                        Some(data) => data.media_list_collection,
                        None => None,
                    },
                    Err(err) => {
                        warn!("anime list query error: {}", err);
                        None
                    }
                };
                let manga_list = match manga_result {
                    Ok(resp) => match resp.data {
                        Some(data) => data.media_list_collection,
                        None => None,
                    },
                    Err(err) => {
                        warn!("manga list query error: {}", err);
                        None
                    }
                };
                ListRetrieved {
                    anime_list,
                    manga_list,
                }
                .into()
            },
        )
    }

    pub fn query_search(
        token: String,
        recognized: recognition::Media,
        oneshot: bool,
    ) -> Command<Message> {
        Command::perform(
            anilist::query_search(Some(token), recognized.title, recognized.media_type),
            move |result| match result {
                Ok(resp) => match resp.data {
                    Some(search_resp) => match search_resp.page.media {
                        Some(results) => SearchResults(results, oneshot).into(),
                        None => NoMessage.into(),
                    },
                    None => NoMessage.into(),
                },
                Err(err) => {
                    warn!("anilist media search error: {}", err);
                    NoMessage.into()
                }
            },
        )
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = App {
            waiting_for_cover: false,
            recognized: None,
            media: None,
            media_cover: None,
            nav: components::Nav::new(),
            page: components::PageContainer::default(),
            user: None,
            updates: anilist::ListUpdateQueue::default(),
        };
        let command = match settings::get_settings().write().unwrap().anilist.token() {
            Some(token) => {
                info!("user token successfully loaded from settings");
                Self::query_user(token.clone())
            }
            None => Command::none(),
        };
        (app, command)
    }

    fn title(&self) -> String {
        "Fubuki".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let mut commands = Vec::new();
        if let Some(msg) = message.handle(self) {
            commands.push(msg);
        }

        if !self.updates.is_waiting() {
            if let Some(media_update) = self.updates.dequeue() {
                self.updates.set_waiting(true);
                if let Some(media) = &self.media {
                    if media.media_id == media_update.media_id {
                        let already_sent = true;
                        commands.push(forward_message(
                            CancelListUpdate(media_update.media_id, already_sent).into(),
                        ));
                    }
                }
                let token = {
                    let settings = settings::get_settings().read().unwrap();
                    settings.anilist.token().clone()
                };
                if let Some(media) = &media_update.media {
                    if let Some(fmt) = &media.media_type {
                        let list = match fmt {
                            anilist::MediaType::Anime => self.page.anime.get_list_mut(),
                            anilist::MediaType::Manga => self.page.manga.get_list_mut(),
                        };
                        if let Some(list) = list {
                            let entry = list.find_entry_by_id_mut(media_update.media_id);
                            if let Some(entry) = entry {
                                *entry = media_update.clone();
                            }
                        }
                    }
                }
                commands.push(Command::perform(
                    anilist::update_media(token, media_update),
                    |result| match result {
                        Ok(resp) => {
                            info!("media update succeeded: {:#?}", resp);
                            MediaUpdateComplete.into()
                        }
                        Err(err) => {
                            warn!("media update failed: {}", err);
                            MediaUpdateComplete.into()
                        }
                    },
                ));
            }
        }
        Command::batch(commands)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_secs(2)).map(|_| DetectMedia.into())
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let nav = self.nav.view(); //.map(move |msg| NavChange(msg).into());
        let page = self.page.view(); //.map(move |msg| PageMessage(msg).into());

        // Scrollable::new(scroll).padding(40).push(Container::new(content.width(Length::Fill).center_x())).into()
        // let media_title = Text::new(&self.media)
        //     .width(iced::Length::Fill);
        // .horizontal_alignment(HorizontalAlignment::Center)
        // .vertical_alignment(VerticalAlignment::Center);

        let content = Column::new()
            // .max_width(400)
            // .max_height(300)
            // .spacing(20)
            .push(nav)
            .push(page);
        // .push(media_title);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container::Background)
            .into()
    }
}

use ui::components::{
    nav::{AnimeListPress, CurrentMediaPress, MangaListPress, SettingsPress},
    page::{
        CancelListUpdate, CoverChange, IncrementMediaProgress, ListFilterTextChange,
        ListGroupSelected, Login, Logout, MediaChange, RefreshLists, SettingChange,
    },
};

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum Message {
    DetectMedia,
    DetectMediaResult,
    MediaFound,
    MediaNotFound,
    Authorized,
    AuthFailed,
    UserFound,
    AvatarRetrieved,
    ListRetrieved,
    CoverRetrieved,
    SearchMedia,
    SearchResults,
    MediaUpdateComplete,

    // Nav
    AnimeListPress,
    MangaListPress,
    CurrentMediaPress,
    SettingsPress,

    // Page
    CoverChange,
    MediaChange,
    RefreshLists,
    Logout,
    Login,
    CancelListUpdate,
    SettingChange,
    ListGroupSelected,
    IncrementMediaProgress,
    ListFilterTextChange,

    NoMessage,
}

pub fn forward_message(msg: Message) -> Command<Message> {
    Command::perform(nothing(msg), |msg| msg)
}

async fn nothing(msg: Message) -> Message {
    msg
}

#[enum_dispatch(Message)]
pub trait Event {
    fn handle(self, app: &mut App) -> Option<Command<Message>>;
}

#[derive(Debug, Clone)]
pub struct DetectMedia;

impl Event for DetectMedia {
    fn handle(self, _app: &mut App) -> Option<Command<Message>> {
        Some(Command::perform(MediaParser::detect_media(), |media| {
            DetectMediaResult(media).into()
        }))
    }
}

#[derive(Debug, Clone)]
pub struct DetectMediaResult(Option<recognition::Media>);

impl Event for DetectMediaResult {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let DetectMediaResult(media) = self;
        if let Some(detected_media) = media {
            match &app.recognized {
                Some(media) => {
                    if *media != detected_media {
                        debug!("detected media {:#?}", detected_media);
                        app.recognized = Some(detected_media.clone());
                        return Some(forward_message(SearchMedia(detected_media, false).into()));
                    } else {
                        return None;
                    }
                }
                None => {
                    debug!("detected media {:#?}", detected_media);
                    app.recognized = Some(detected_media.clone());
                    return Some(forward_message(SearchMedia(detected_media, false).into()));
                }
            }
        }
        Some(forward_message(MediaNotFound.into()))
    }
}

#[derive(Debug, Clone)]
pub struct MediaFound(anilist::MediaList, recognition::Media, bool);

impl Event for MediaFound {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let MediaFound(media, detected_media, needs_update) = self;
        let (cover_url, needs_fetch) = match &media.media {
            Some(media) => {
                let url = media.cover_image_url();
                match &app.media {
                    Some(old_media) => match &old_media.media {
                        Some(old_media) => {
                            if url == old_media.cover_image_url() {
                                (url, false)
                            } else {
                                (url, true)
                            }
                        }
                        None => (url, true),
                    },
                    None => (url, true),
                }
            }
            None => (None, false),
        };

        app.media = Some(media.clone());
        app.recognized = Some(detected_media.clone());

        let msg = MediaChange(Some(media.clone()), Some(detected_media), needs_update).into();
        let mut commands = vec![forward_message(msg)];

        if let Some(cover_url) = cover_url {
            if !app.waiting_for_cover && needs_fetch {
                app.waiting_for_cover = true;
                commands.push(Command::perform(
                    ui::util::fetch_image(cover_url),
                    |result| {
                        let handle = match result {
                            Ok(handle) => Some(handle),
                            Err(err) => {
                                warn!("could not get cover: {}", err);
                                None
                            }
                        };
                        CoverRetrieved(handle).into()
                    },
                ));
            }
        } else {
            let msg = CoverChange(None).into();
            commands.push(forward_message(msg));
        }

        if needs_update {
            app.updates.enqueue(media);
        } else {
            debug!("update not needed for media id {}", media.media_id);
        }
        return Some(Command::batch(commands));
    }
}

#[derive(Debug, Clone)]
pub struct MediaNotFound;

impl Event for MediaNotFound {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let media_id = app.media.as_ref()?.media_id;
        app.recognized = None;
        app.media = None;
        app.media_cover = None;
        let index = app.updates.find_index(media_id);
        if let Some(index) = index {
            app.updates.remove(index);
        }
        Some(forward_message(MediaChange(None, None, false).into()))
    }
}

#[derive(Debug, Clone)]
pub struct SearchMedia(recognition::Media, bool);

impl Event for SearchMedia {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let SearchMedia(recognized, oneshot) = self;
        app.recognized = Some(recognized.clone());
        let token = {
            let settings = settings::get_settings().read().unwrap();
            settings.anilist.token().clone()
        }?;
        Some(App::query_search(token, recognized, oneshot))
    }
}

#[derive(Debug, Clone)]
pub struct SearchResults(Vec<Option<anilist::Media>>, bool);

impl Event for SearchResults {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let SearchResults(results, oneshot) = self;
        let results: Vec<Option<&anilist::Media>> = results
            .iter()
            .filter_map(|m| m.as_ref())
            .map(|m| Some(m))
            .collect();
        let mut recognized = app.recognized.clone()?;
        let mut id =
            anilist::MediaListCollection::best_id_for_search(&results, &recognized.title, oneshot)?;
        let progress = {
            let list = match recognized.media_type {
                anilist::MediaType::Anime => app.page.anime.get_list(),
                anilist::MediaType::Manga => app.page.manga.get_list(),
            };
            let progress = match recognized.media_type {
                anilist::MediaType::Anime => {
                    match list {
                        Some(list) => match recognized.progress {
                            Some(new_progress) => {
                                let mut offset_progress =
                                    list.compute_progress_offset_by_id(id, new_progress as i32);

                                if let Some(offset) = offset_progress {
                                    if offset < 0 {
                                        // try to find offset for immediate sequel
                                        let sequel_offset = list
                                            .compute_progress_offset_for_sequel(
                                                id,
                                                new_progress as i32,
                                            );
                                        match sequel_offset {
                                            Some((offset, sequel_id)) => {
                                                offset_progress = Some(offset);
                                                id = sequel_id
                                            }
                                            None => {}
                                        }
                                    }
                                }

                                offset_progress
                            }
                            None => None,
                        },
                        None => None,
                    }
                }
                _ => None,
            };

            progress
        };

        let list = match recognized.media_type {
            anilist::MediaType::Anime => app.page.anime.get_list_mut(),
            anilist::MediaType::Manga => app.page.manga.get_list_mut(),
        }?;

        let entry = list.find_entry_by_id_mut(id);
        match entry {
            Some(media) => {
                // Check if the detected progress is larger than the media's maximum number of episodes/chapters
                // This is most likely an nth season where the count rolled over
                if let Some(progress) = progress {
                    if let Some(recognized_progress) = recognized.progress {
                        if progress > 0 && progress < recognized_progress as i32 {
                            debug!(
                                "offset progress of media {} to {} instead of {}",
                                media.media_id, progress, recognized_progress
                            );
                            recognized.progress = Some(progress as f64);
                        } else {
                            recognized.progress = None;
                            warn!(
                                "detected progress offset error for media {}, {} became {}",
                                media.media_id, recognized_progress, progress
                            );
                        }
                    } else {
                        warn!("no recognized progress from {:#?}", recognized);
                    }
                } else {
                    debug!("progress offset was None for media id {}", media.media_id);
                }

                // Clone the media so we only mutate the entry in the user's list
                // when the request is going to be sent, since the update can be cancelled
                let mut media_copy = media.clone();
                let needs_update =
                    media_copy.update_progress(recognized.progress, recognized.progress_volumes);
                Some(forward_message(
                    MediaFound(media_copy, recognized, needs_update).into(),
                ))
            }
            None => {
                debug!("could not find media in list");
                Some(forward_message(MediaNotFound.into()))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Authorized(String);

impl Event for Authorized {
    fn handle(self, _app: &mut App) -> Option<Command<Message>> {
        let Authorized(token) = self;
        let mut settings = settings::get_settings().write().unwrap();
        settings.anilist.save_token(token.as_str());
        if let Err(err) = settings.anilist.save() {
            warn!("couldn't save token: {}", err);
            Some(forward_message(AuthFailed.into()))
        } else {
            Some(App::query_user(token))
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthFailed;

impl Event for AuthFailed {
    fn handle(self, _app: &mut App) -> Option<Command<Message>> {
        Some(forward_message(Logout.into()))
    }
}

#[derive(Debug, Clone)]
pub struct UserFound(anilist::User);

impl Event for UserFound {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let UserFound(user) = self;
        info!("retrieved user: {}", user.id);
        app.user = Some(user);

        app.page.settings.logged_in = true;
        Some(Command::perform(
            ui::util::fetch_image(app.user.as_ref()?.get_avatar_url()?),
            |result| match result {
                Ok(handle) => AvatarRetrieved(handle).into(),
                Err(err) => {
                    warn!("failed to get avatar: {}", err);
                    NoMessage.into()
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct AvatarRetrieved(iced::image::Handle);

impl Event for AvatarRetrieved {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let AvatarRetrieved(handle) = self;
        info!("retrieved avatar");
        app.nav.set_avatar(Some(handle));

        let settings = settings::get_settings().read().unwrap();
        let token = settings.anilist.token().as_ref()?;
        Some(App::query_user_lists(token.clone(), app.user.as_ref()?.id))
    }
}

#[derive(Debug, Clone)]
pub struct ListRetrieved {
    anime_list: Option<anilist::MediaListCollection>,
    manga_list: Option<anilist::MediaListCollection>,
}

impl Event for ListRetrieved {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        app.page.anime.set_list(self.anime_list);
        app.page.manga.set_list(self.manga_list);
        info!(
            "anime list was retrieved? {}",
            app.page.anime.get_list().is_some()
        );
        info!(
            "manga list was retrieved? {}",
            app.page.manga.get_list().is_some()
        );
        None
    }
}

#[derive(Debug, Clone)]
pub struct CoverRetrieved(Option<iced::image::Handle>);

impl Event for CoverRetrieved {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        let CoverRetrieved(cover) = self;
        app.waiting_for_cover = false;
        app.media_cover = cover.clone();
        Some(forward_message(CoverChange(cover).into()))
    }
}

#[derive(Debug, Clone)]
pub struct MediaUpdateComplete;

impl Event for MediaUpdateComplete {
    fn handle(self, app: &mut App) -> Option<Command<Message>> {
        app.updates.set_waiting(false);
        None
    }
}

#[derive(Debug, Clone)]
pub struct NoMessage;

impl Event for NoMessage {
    fn handle(self, _app: &mut App) -> Option<Command<Message>> {
        None
    }
}
