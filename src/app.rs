use crate::*;
use recognition::MediaParser;
use ui::{components, style};
use enum_dispatch::enum_dispatch;
use std::fmt::Debug;
use iced::{
    executor,
    time,
    Application,
    Column, // Text, // HorizontalAlignment, VerticalAlignment,
    Command,
    Container,
    Element,
    Length,
    Subscription,
    image,
    button,
};

#[derive(Default)]
pub struct App {
    pub waiting_for_cover: bool,
    pub media: Option<anilist::MediaList>,
    pub media_cover: Option<image::Handle>,
    pub nav: components::Nav,
    pub page: components::Page,
    pub user: Option<anilist::User>,
    pub anime_list: Option<anilist::MediaListCollection>,
    pub manga_list: Option<anilist::MediaListCollection>,
}

impl App {
    fn query_user(token: String) -> Command<Message> {
        Command::perform(
            anilist::query_user(Some(token)),
            |result| match result {
                Ok(resp) => {
                    if let Some(viewer_resp) = resp.data {
                        if let Some(user) = viewer_resp.viewer {
                            return UserFound(user).into();
                        }
                    }
                    AuthFailed.into()
                }
                Err(err) => {
                    eprintln!("user query failed: {}", err);
                    AuthFailed.into()
                }
            },
        )
    }

    fn auth() -> Command<Message> {
        Command::perform(anilist::auth(), |result| match result {
            Ok(token) => Authorized(token).into(),
            Err(err) => {
                println!("authorization failed: {}", err);
                AuthFailed.into()
            }
        })
    }

    fn query_user_lists(token: String, user_id: i32) -> Command<Message> {
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
                }.into()
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
            media: None,
            media_cover: None,
            nav: components::Nav::new(),
            page: components::Page::default(),
            user: None,
            anime_list: None,
            manga_list: None,
        };
        let command = match settings::get_settings().write().unwrap().anilist.token() {
            Some(token) => {
                println!("already authorized");
                Self::query_user(token.clone())
            }
            None => Self::auth(),
        };
        (app, command)
    }

    fn title(&self) -> String {
        "Fubuki".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        message.handle(self)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_secs(2)).map(|_| SearchMedia.into())
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let nav = self.nav.view().map(move |msg| NavChange(msg).into());
        let page = self.page.view().map(move |msg| Page(msg).into());

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

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum Message {
    SearchMedia,
    SearchResult,
    MediaFound,
    MediaNotFound,
    NavChange,
    Page,
    Authorized,
    AuthFailed,
    UserFound,
    AvatarRetrieved,
    ListRetrieved,
    CoverRetrieved,
}

#[enum_dispatch(Message)]
pub trait Event: Debug + Send {
    fn handle(self, app: &mut App) -> Command<Message>;
}

#[derive(Debug, Clone)]
pub struct SearchMedia;

impl Event for SearchMedia {
    fn handle(self, _app: &mut App) -> Command<Message> {
        Command::perform(MediaParser::detect_media(), |media| SearchResult(media).into())
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult(Option<recognition::Media>);

impl Event for SearchResult {
    fn handle(self, app: &mut App) -> Command<Message> {
        let media = self.0;
        if let Some(detected_media) = media {
            println!("detected media {:#?}", detected_media);

            let media = {
                let list = match detected_media.media_type {
                    anilist::MediaType::Anime => &mut app.anime_list,
                    anilist::MediaType::Manga => &mut app.manga_list,
                };

                match list {
                    Some(list) => list.search_for_title(&detected_media.title),
                    None => None,
                }
            };
            
            if let Some(media) = media {
                let needs_update = media.update_progress(detected_media.progress, detected_media.progress_volumes);
                let media = media.clone();
                return app.update(MediaFound(media, needs_update).into());
            } else {
                return app.update(MediaNotFound.into());
            }
        } else {
            return app.update(MediaNotFound.into())
        }
        // Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct MediaFound(anilist::MediaList, bool);

impl Event for MediaFound {
    fn handle(self, app: &mut App) -> Command<Message> {
        let media = self.0;
        let needs_update = self.1;
        let cover_url = match &media.media {
            Some(media) => media.cover_image_url(),
            None => None,
        };
        app.media = Some(media.clone());
        match app.page {
            components::page::Page::CurrentMedia { current: _, cover: _, default_cover: _ } => {
                app.page.update(components::page::Message::MediaFound(media.clone()));
            },
            _ => {}
        }
        let mut commands = Vec::new();
        if let Some(cover_url) = cover_url {
            if let None = app.media_cover {
                if !app.waiting_for_cover {
                    app.waiting_for_cover = true;
                    commands.push(Command::perform(ui::util::fetch_image(cover_url), |result| {
                        let handle = match result {
                            Ok(handle) => {
                                Some(handle)
                            },
                            Err(err) => { 
                                eprintln!("could not get cover {}", err);
                                None
                            },
                        };
                        CoverRetrieved(handle).into()
                    }));
                }
            }
        } else {
            app.page.update(components::page::Message::CoverChange(None));
        }
        let token = {
            let settings = settings::get_settings().read().unwrap();
            settings.anilist.token().clone()
        };
        if needs_update {
            println!("sending update");
            commands.push(Command::perform(anilist::update_media(token, media), |result| match result {
                Ok(resp) => {
                    println!("media update succeeded: {:#?}", resp);
                    AuthFailed.into()
                },
                Err(err) => {
                    println!("media update failed: {}", err);
                    AuthFailed.into()
                }
            }));
        } else {
            println!("update not needed")
        }
        return Command::batch(commands);
    }
}

#[derive(Debug, Clone)]
pub struct MediaNotFound;

impl Event for MediaNotFound {
    fn handle(self, app: &mut App) -> Command<Message> {
        app.media = None;
        app.media_cover = None;
        match app.page {
            components::page::Page::CurrentMedia { current: _, cover: _, default_cover: _  } => {
                app.page.update(components::page::Message::MediaNotFound);
            }
             _ => {},
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct NavChange(components::nav::Message);

impl Event for NavChange {
    fn handle(self, app: &mut App) -> Command<Message> {
        let msg = self.0;
        match msg {
            components::nav::Message::CurrentMediaPress { selected } => {
                if !selected {
                    println!("pressed media");
                    app.nav.update(msg);
                    match app.page {
                        components::Page::Settings { refresh_list_state: _ } => {
                            app.page = components::Page::CurrentMedia {
                                current: app.media.clone(),
                                cover: app.media_cover.clone(),
                                default_cover: image::Handle::from("./res/cover_default.jpg"),
                            };
                        }
                        _ => {}
                    }
                }
            },
            components::nav::Message::SettingsPress { selected } => {
                if !selected {
                    println!("pressed settings");
                    app.nav.update(msg);
                    match app.page {
                        components::Page::CurrentMedia { current: _, cover: _, default_cover: _ } => {
                            app.page = components::Page::Settings { refresh_list_state: button::State::default() };
                        }
                        _ => {}
                    }
                }
            },
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Page(components::page::Message);

impl Event for Page {
    fn handle(self, app: &mut App) -> Command<Message> {
        let msg = self.0;
        match msg {
            components::page::Message::RefreshLists => {
                let settings = settings::SETTINGS.read().unwrap();
                let token = settings.anilist.token().clone();
                if let Some(token) = token {
                    if let Some(user) = &app.user {
                        return App::query_user_lists(token, user.id);
                    }
                }
            }
            _ => {}
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Authorized(String);

impl Event for Authorized {
    fn handle(self, _app: &mut App) -> Command<Message> {
        let token = self.0;
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
        let user = self.0;
        app.user = Some(user);
        println!("got user {:#?}", app.user);

        if let Some(user) = &app.user {
            let url = user.get_avatar_url();
            if let Some(url) = url {
                return Command::perform(ui::util::fetch_image(url), |result| match result {
                    Ok(handle) => {
                        AvatarRetrieved(handle).into()
                    },
                    Err(err) => {
                        eprintln!("failed to get avatar {}", err);
                        AuthFailed.into()
                    },
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
        let handle = self.0;
        println!("got avatar");
        app.nav.set_avatar(handle);
                
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
        app.anime_list = self.anime_list;
        app.manga_list = self.manga_list;
        println!("got the list response");
        println!("  anime list is some? {}", app.anime_list.is_some());
        println!("  manga list is some? {}", app.manga_list.is_some());
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct CoverRetrieved(Option<image::Handle>);

impl Event for CoverRetrieved {
    fn handle(self, app: &mut App) -> Command<Message> {
        let cover = self.0;
        app.waiting_for_cover = false;
        app.media_cover = cover.clone();
        match app.page {
            components::page::Page::CurrentMedia{ current: _, cover: _, default_cover: _ } => {
                app.page.update(components::page::Message::CoverChange(cover));
            }
            _ => {},
        }
        Command::none()
    }
}