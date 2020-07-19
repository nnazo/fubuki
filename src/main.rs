// #[macro_use] extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
#[macro_use]
extern crate lazy_static;

pub mod anilist;
pub mod recognition;
pub mod settings;
pub mod ui {
    pub mod components;
    pub mod style;
}
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
    MediaFound(String),
    MediaNotFound,
    NavChange(components::nav::Message),
    Page(components::page::Message),
    Authorized(String),
    AuthFailed,
    UserFound(anilist::User),
    ListRetrieved {
        anime_list: Option<anilist::MediaListCollection>,
        manga_list: Option<anilist::MediaListCollection>,
    },
}

#[derive(Default)]
struct App {
    media: String,
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
            media: "None Found".to_string(),
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
                let settings = settings::get_settings().read().unwrap();
                let token = settings.anilist.token();
                if let Some(user) = &self.user {
                    if let Some(token) = token {
                        return Self::query_user_lists(token.clone(), user.id);
                    }
                }
            }
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

                    let list = match detected_media.media_type {
                        anilist::MediaType::Anime => &mut self.anime_list,
                        anilist::MediaType::Manga => &mut self.manga_list,
                    };

                    let media = match list {
                        Some(list) => list.search_title(&detected_media.title),
                        None => None,
                    };

                    if let Some(media) = media {
                        let needs_update = media.update_progress(detected_media.progress, detected_media.progress_volumes);
                        let media = media.clone();
                        self.update(Message::MediaFound(format!("{:#?}", media.clone())));
                        let token = {
                            let settings = settings::get_settings().read().unwrap();
                            settings.anilist.token().clone()
                        };
                        if needs_update {
                            println!("sending update");
                            return Command::perform(anilist::update_media(token, media), |result| match result {
                                Ok(resp) => {
                                    println!("media update succeeded: {:#?}", resp);
                                    Message::AuthFailed
                                },
                                Err(err) => {
                                    println!("media update failed: {}", err);
                                    Message::AuthFailed
                                }
                            });
                        } else {
                            println!("update not needed")
                        }
                    } else {
                        self.update(Message::MediaNotFound);
                    }
                } else {
                    self.update(Message::MediaNotFound);
                }
            }
            Message::MediaFound(title) => {
                self.media = title;
                match self.page {
                    components::page::Page::CurrentMedia { current: _ } => self
                        .page
                        .update(components::page::Message::MediaFound(self.media.clone())),
                    _ => {}
                }
            }
            Message::MediaNotFound => {
                // println!("debug token: {:#?}", self.settings.anilist);
                self.media = "None Found".to_string();
                match self.page {
                    components::page::Page::CurrentMedia { current: _ } => {
                        self.page.update(components::page::Message::MediaNotFound)
                    }
                    _ => {}
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
                                };
                            }
                            _ => {}
                        }
                    }
                }
                components::nav::Message::SettingsPress { selected } => {
                    if !selected {
                        println!("pressed settings");
                        self.nav.update(msg);
                        match self.page {
                            components::Page::CurrentMedia { current: _ } => {
                                self.page = components::Page::Settings;
                            }
                            _ => {}
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
