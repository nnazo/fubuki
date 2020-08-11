use crate::*;
use enum_dispatch::enum_dispatch;
use iced::{
    executor,
    image,
    time,
    Application,
    Column, // Text, // HorizontalAlignment, VerticalAlignment,
    Command,
    Container,
    Element,
    Length,
    Subscription,
};
use recognition::MediaParser;
use std::fmt::Debug;
use ui::{components, style};

#[derive(Default)]
pub struct App {
    pub waiting_for_cover: bool,
    pub recognized: Option<recognition::Media>,
    pub media: Option<anilist::MediaList>,
    pub media_cover: Option<image::Handle>,
    pub nav: components::Nav,
    pub page: components::PageContainer,
    pub user: Option<anilist::User>,
    // pub anime_list: Option<anilist::MediaListCollection>,
    // pub manga_list: Option<anilist::MediaListCollection>,
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
                eprintln!("user query failed: {}", err);
                NoMessage.into()
            }
        })
    }

    pub fn auth() -> Command<Message> {
        Command::perform(anilist::auth(), |result| match result {
            Ok(token) => Authorized(token).into(),
            Err(err) => {
                println!("authorization failed: {}", err);
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
                        eprintln!("query err: {}", err);
                        None
                    }
                };
                let manga_list = match manga_result {
                    Ok(resp) => match resp.data {
                        Some(data) => data.media_list_collection,
                        None => None,
                    },
                    Err(err) => {
                        eprintln!("query err: {}", err);
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
                    eprintln!("search err: {}", err);
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
            // anime_list: None,
            // manga_list: None,
            updates: anilist::ListUpdateQueue::default(),
        };
        let command = match settings::get_settings().write().unwrap().anilist.token() {
            Some(token) => {
                println!("already authorized");
                Self::query_user(token.clone())
            }
            None =>
            /*Self::auth()*/
            {
                Command::none()
            }
        };
        (app, command)
    }

    fn title(&self) -> String {
        "Fubuki".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let mut commands = vec![message.handle(self)];
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
                            println!("media update succeeded: {:#?}", resp);
                            MediaUpdateComplete.into()
                        }
                        Err(err) => {
                            println!("media update failed: {}", err);
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
    fn handle(self, app: &mut App) -> Command<Message>;
}

#[derive(Debug, Clone)]
pub struct DetectMedia;

impl Event for DetectMedia {
    fn handle(self, _app: &mut App) -> Command<Message> {
        Command::perform(MediaParser::detect_media(), |media| {
            DetectMediaResult(media).into()
        })
    }
}

#[derive(Debug, Clone)]
pub struct DetectMediaResult(Option<recognition::Media>);

impl Event for DetectMediaResult {
    fn handle(self, app: &mut App) -> Command<Message> {
        let DetectMediaResult(media) = self;
        if let Some(detected_media) = media {
            println!("detected media {:#?}", detected_media);
            match &app.recognized {
                Some(media) => {
                    if *media != detected_media {
                        app.recognized = Some(detected_media.clone());
                        return forward_message(SearchMedia(detected_media, false).into());
                    } else {
                        return Command::none();
                    }
                }
                None => {
                    app.recognized = Some(detected_media.clone());
                    return forward_message(SearchMedia(detected_media, false).into());
                }
            }

            // let media = {
            //     let list = match detected_media.media_type {
            //         anilist::MediaType::Anime => &mut app.anime_list,
            //         anilist::MediaType::Manga => &mut app.manga_list,
            //     };

            //     match list {
            //         Some(list) => list.search_for_title(&detected_media.title),
            //         None => None,
            //     }
            // };

            // if let Some(media) = media {
            //     // TODO: Check if the detected progress is larger than the media's maximum number of episodes/chapters
            //     // This is most likely an nth season where the count rolled over
            //     let needs_update =
            //         media.update_progress(detected_media.progress, detected_media.progress_volumes);
            //     let media = media.clone();
            //     return app.update(MediaFound(media, detected_media, needs_update).into());
            // } else {
            //     return app.update(MediaNotFound.into());
            // }
        }
        forward_message(MediaNotFound.into())
    }
}

#[derive(Debug, Clone)]
pub struct MediaFound(anilist::MediaList, recognition::Media, bool);

impl Event for MediaFound {
    fn handle(self, app: &mut App) -> Command<Message> {
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
                                eprintln!("could not get cover {}", err);
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
            println!("update not needed");
        }
        return Command::batch(commands);
    }
}

#[derive(Debug, Clone)]
pub struct MediaNotFound;

impl Event for MediaNotFound {
    fn handle(self, app: &mut App) -> Command<Message> {
        let mut commands = Vec::new();
        if let Some(media) = &app.media {
            commands.push(forward_message(MediaChange(None, None, false).into()));
            commands.push(forward_message(CoverChange(None).into()));
            let index = app.updates.find_index(media.media_id);
            if let Some(index) = index {
                app.updates.remove(index);
            }
        }
        app.recognized = None;
        app.media = None;
        app.media_cover = None;
        Command::batch(commands)
    }
}

#[derive(Debug, Clone)]
pub struct SearchMedia(recognition::Media, bool);

impl Event for SearchMedia {
    fn handle(self, app: &mut App) -> Command<Message> {
        let SearchMedia(recognized, oneshot) = self;
        let token = {
            let settings = settings::get_settings().read().unwrap();
            settings.anilist.token().clone()
        };
        match token {
            Some(tok) => {
                app.recognized = Some(recognized.clone());
                App::query_search(tok, recognized, oneshot)
            }
            None => Command::none(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResults(Vec<Option<anilist::Media>>, bool);

impl Event for SearchResults {
    fn handle(self, app: &mut App) -> Command<Message> {
        let SearchResults(results, oneshot) = self;
        let results: Vec<Option<&anilist::Media>> = results
            .iter()
            .filter_map(|m| m.as_ref())
            .map(|m| Some(m))
            .collect();
        if let Some(mut recognized) = app.recognized.clone() {
            let id = anilist::MediaListCollection::best_id_for_search(
                &results,
                &recognized.title,
                oneshot,
            );
            if let Some(mut id) = id {
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
                                        let mut offset_progress = list
                                            .compute_progress_offset_by_id(id, new_progress as i32);

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
                };
                if let Some(list) = list {
                    let entry = list.find_entry_by_id_mut(id);
                    if let Some(media) = entry {
                        // Check if the detected progress is larger than the media's maximum number of episodes/chapters
                        // This is most likely an nth season where the count rolled over
                        if let Some(progress) = progress {
                            if let Some(recognized_progress) = recognized.progress {
                                if progress > 0 && progress < recognized_progress as i32 {
                                    println!(
                                        "offset progress to {} instead of {}",
                                        progress, recognized_progress
                                    );
                                    recognized.progress = Some(progress as f64);
                                } else {
                                    recognized.progress = None;
                                    println!(
                                        "something went wrong with detected progress, {} became {}",
                                        recognized_progress, progress
                                    );
                                }
                            } else {
                                println!("no recognized progress");
                            }
                        } else {
                            println!("progress offset was None");
                        }

                        // Clone the media so we only mutate the entry in the user's list
                        // when the request is going to be sent, since the update can be cancelled
                        let mut media_copy = media.clone();
                        let needs_update = media_copy
                            .update_progress(recognized.progress, recognized.progress_volumes);
                        return forward_message(
                            MediaFound(media_copy, recognized, needs_update).into(),
                        );
                    } else {
                        println!("could not find media in list");
                        return forward_message(MediaNotFound.into());
                    }
                }
            }
        }

        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Authorized(String);

impl Event for Authorized {
    fn handle(self, _app: &mut App) -> Command<Message> {
        let Authorized(token) = self;
        let mut settings = settings::get_settings().write().unwrap();
        settings.anilist.save_token(token.as_str());
        if let Err(err) = settings.anilist.save() {
            println!("couldn't save token: {}", err);
        }
        App::query_user(token)
    }
}

#[derive(Debug, Clone)]
pub struct AuthFailed;

impl Event for AuthFailed {
    fn handle(self, _app: &mut App) -> Command<Message> {
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct UserFound(anilist::User);

impl Event for UserFound {
    fn handle(self, app: &mut App) -> Command<Message> {
        let UserFound(user) = self;
        app.user = Some(user);
        println!("got user {:#?}", app.user);

        app.page.settings.logged_in = true;

        if let Some(user) = &app.user {
            let url = user.get_avatar_url();
            if let Some(url) = url {
                return Command::perform(ui::util::fetch_image(url), |result| match result {
                    Ok(handle) => AvatarRetrieved(handle).into(),
                    Err(err) => {
                        eprintln!("failed to get avatar {}", err);
                        NoMessage.into()
                    }
                });
            }
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct AvatarRetrieved(image::Handle);

impl Event for AvatarRetrieved {
    fn handle(self, app: &mut App) -> Command<Message> {
        let AvatarRetrieved(handle) = self;
        println!("got avatar");
        app.nav.set_avatar(Some(handle));

        let settings = settings::get_settings().read().unwrap();
        let token = settings.anilist.token();
        if let Some(user) = &app.user {
            if let Some(token) = token {
                return App::query_user_lists(token.clone(), user.id);
            }
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct ListRetrieved {
    anime_list: Option<anilist::MediaListCollection>,
    manga_list: Option<anilist::MediaListCollection>,
}

impl Event for ListRetrieved {
    fn handle(self, app: &mut App) -> Command<Message> {
        app.page.anime.set_list(self.anime_list);
        app.page.manga.set_list(self.manga_list);
        println!("got the list response");
        println!(
            "  anime list is some? {}",
            app.page.anime.get_list().is_some()
        );
        println!(
            "  manga list is some? {}",
            app.page.manga.get_list().is_some()
        );
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct CoverRetrieved(Option<image::Handle>);

impl Event for CoverRetrieved {
    fn handle(self, app: &mut App) -> Command<Message> {
        let CoverRetrieved(cover) = self;
        app.waiting_for_cover = false;
        app.media_cover = cover.clone();
        forward_message(CoverChange(cover).into())
    }
}

#[derive(Debug, Clone)]
pub struct MediaUpdateComplete;

impl Event for MediaUpdateComplete {
    fn handle(self, app: &mut App) -> Command<Message> {
        app.updates.set_waiting(false);
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct NoMessage;

impl Event for NoMessage {
    fn handle(self, _app: &mut App) -> Command<Message> {
        Command::none()
    }
}
