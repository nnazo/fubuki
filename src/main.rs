// #[macro_use] extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
#[macro_use]
extern crate lazy_static;

pub mod anilist;
pub mod recognition;
pub mod settings;
pub mod ui;
use recognition::MediaParser;
use ui::{components, style};

use iced::{
    executor,
    time,
    Application,
    Column, // Text, // HorizontalAlignment, VerticalAlignment,
    Command,
    Container,
    Element,
    Length,
    Settings,
    Subscription,
    image,
};
// use std::collections::HashMap;
// use graphql_client::GraphQLQuery;

//#![windows_subsystem = "windows"] // Tells windows compiler not to show console window

fn main() {
    App::run(Settings::default());
}

#[derive(Debug, Clone)]
enum Message {
    SearchMedia,
    SearchResult(Option<recognition::Media>),
    MediaFound(anilist::MediaList, bool),
    MediaNotFound,
    NavChange(components::nav::Message),
    Page(components::page::Message),
    Authorized(String),
    AuthFailed,
    UserFound(anilist::User),
    AvatarRetrieved(iced::image::Handle),
    ListRetrieved {
        anime_list: Option<anilist::MediaListCollection>,
        manga_list: Option<anilist::MediaListCollection>,
    },
    CoverRetrieved(Option<image::Handle>),
}

#[derive(Default)]
struct App {
    waiting_for_cover: bool,
    media: Option<anilist::MediaList>,
    media_cover: Option<image::Handle>,
    nav: components::Nav,
    page: components::Page,
    user: Option<anilist::User>,
    anime_list: Option<anilist::MediaListCollection>,
    manga_list: Option<anilist::MediaListCollection>,
}

impl App {
    fn query_user(token: String) -> Command<Message> {
        Command::perform(
            anilist::query_user(Some(token)),
            |result| match result {
                Ok(resp) => {
                    if let Some(viewer_resp) = resp.data {
                        if let Some(user) = viewer_resp.viewer {
                            return Message::UserFound(user);
                        }
                    }
                    Message::AuthFailed
                }
                Err(err) => {
                    eprintln!("user query failed: {}", err);
                    Message::AuthFailed
                }
            },
        )
    }

    fn auth() -> Command<Message> {
        Command::perform(anilist::auth(), |result| match result {
            Ok(token) => Message::Authorized(token),
            Err(err) => {
                println!("authorization failed: {}", err);
                Message::AuthFailed
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
                Message::ListRetrieved {
                    anime_list,
                    manga_list,
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
        match message {
            Message::Authorized(token) => {
                let mut settings = settings::get_settings().write().unwrap();
                settings.anilist.save_token(token.as_str());
                if let Err(err) = settings.anilist.save() {
                    println!("couldn't save token: {}", err);
                }
                return Self::query_user(token);
            }
            Message::UserFound(user) => {
                self.user = Some(user);
                println!("got user {:#?}", self.user);

                if let Some(user) = &self.user {
                    let url = user.get_avatar_url();
                    if let Some(url) = url {
                        return Command::perform(ui::util::fetch_image(url), |result| match result {
                            Ok(handle) => {
                                Message::AvatarRetrieved(handle)
                            },
                            Err(err) => {
                                eprintln!("failed to get avatar {}", err);
                                Message::AuthFailed
                            },
                        });
                    }
                }
            },
            Message::AvatarRetrieved(handle) => {
                println!("got avatar");
                self.nav.set_avatar(handle);
                
                let settings = settings::get_settings().read().unwrap();
                let token = settings.anilist.token();
                if let Some(user) = &self.user {
                    if let Some(token) = token {
                        return Self::query_user_lists(token.clone(), user.id);
                    }
                }
            },
            Message::AuthFailed => {}
            Message::ListRetrieved {
                anime_list,
                manga_list,
            } => {
                self.anime_list = anime_list;
                self.manga_list = manga_list;
                println!("got the list response");
                println!("  anime list is some? {}", self.anime_list.is_some());
                println!("  manga list is some? {}", self.manga_list.is_some());
            }
            Message::SearchMedia => {
                return Command::perform(MediaParser::detect_media(), Message::SearchResult);
            }
            Message::SearchResult(result) => {
                if let Some(detected_media) = result {
                    println!("detected media {:#?}", detected_media);

                    let media = {
                        let list = match detected_media.media_type {
                            anilist::MediaType::Anime => &mut self.anime_list,
                            anilist::MediaType::Manga => &mut self.manga_list,
                        };
    
                        match list {
                            Some(list) => list.search_for_title(&detected_media.title),
                            None => None,
                        }
                    };
                    
                    if let Some(media) = media {
                        let needs_update = media.update_progress(detected_media.progress, detected_media.progress_volumes);
                        let media = media.clone();
                        return self.update(Message::MediaFound(media, needs_update));
                    } else {
                        self.update(Message::MediaNotFound);
                    }
                } else {
                    self.update(Message::MediaNotFound);
                }
            }
            Message::MediaFound(media, needs_update) => {
                let cover_url = match &media.media {
                    Some(media) => media.cover_image_url(),
                    None => None,
                };
                self.media = Some(media.clone());
                match self.page {
                    components::page::Page::CurrentMedia { current: _, cover: _ } => {
                        self.page.update(components::page::Message::MediaFound(media.clone()));
                    },
                    _ => {}
                }
                let mut commands = Vec::new();
                if let Some(cover_url) = cover_url {
                    if let None = self.media_cover {
                        if !self.waiting_for_cover {
                            self.waiting_for_cover = true;
                            println!("i am here");
                            commands.push(Command::perform(ui::util::fetch_image(cover_url), |result| {
                                let handle = match result {
                                    Ok(handle) => {
                                        println!("got cover image");
                                        Some(handle)
                                    },
                                    Err(err) => { 
                                        eprintln!("could not get cover {}", err);
                                        None
                                    },
                                };
                                Message::CoverRetrieved(handle)
                            }));
                        }
                    }
                } else {
                    println!("no cover");
                    self.page.update(components::page::Message::CoverChange(None));
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
                            Message::AuthFailed
                        },
                        Err(err) => {
                            println!("media update failed: {}", err);
                            Message::AuthFailed
                        }
                    }));
                } else {
                    println!("update not needed")
                }
                return Command::batch(commands);
            }
            Message::MediaNotFound => {
                self.media = None;
                self.media_cover = None;
                match self.page {
                    components::page::Page::CurrentMedia { current: _, cover: _  } => {
                        self.page.update(components::page::Message::MediaNotFound);
                    }
                    _ => {},
                }
            }
            Message::CoverRetrieved(cover) => {
                println!("am i ever here..");
                self.waiting_for_cover = false;
                self.media_cover = cover.clone();
                match self.page {
                    components::page::Page::CurrentMedia{ current: _, cover: _ } => {
                        println!("uh");
                        self.page.update(components::page::Message::CoverChange(cover));
                    }
                    _ => {println!("none from cover retrieve")},
                }
            }
            Message::NavChange(msg) => match msg {
                components::nav::Message::CurrentMediaPress { selected } => {
                    if !selected {
                        println!("pressed media");
                        self.nav.update(msg);
                        match self.page {
                            components::Page::Settings => {
                                self.page = components::Page::CurrentMedia {
                                    current: self.media.clone(),
                                    cover: self.media_cover.clone(),
                                };
                            }
                            _ => {}
                        }
                    }
                },
                components::nav::Message::SettingsPress { selected } => {
                    if !selected {
                        println!("pressed settings");
                        self.nav.update(msg);
                        match self.page {
                            components::Page::CurrentMedia { current: _, cover: _  } => {
                                self.page = components::Page::Settings;
                            }
                            _ => {}
                        }
                    }
                },
                components::nav::Message::RefreshLists => {
                    let settings = settings::SETTINGS.read().unwrap();
                    let token = settings.anilist.token().clone();
                    if let Some(token) = token {
                        if let Some(user) = &self.user {
                            return Self::query_user_lists(token, user.id);
                        }
                    }
                }
            },
            Message::Page(msg) => {
                // i could use this to listen to list update events potentially ..?
                match msg {
                    // components::page::Message::MediaFound
                    _ => {}
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_secs(2)).map(|_| Message::SearchMedia)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let nav = self.nav.view().map(move |msg| Message::NavChange(msg));
        let page = self.page.view().map(move |msg| Message::Page(msg));

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
