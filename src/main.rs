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
    SearchResult(Option<String>),
    MediaFound(String),
    MediaNotFound,
    NavChange(components::nav::Message),
    Page(components::page::Message),
    Authorized(String),
    AuthFailed,
}

#[derive(Default)]
struct App {
    // settings: settings::Settings,
    media: String,
    // parser: fubuki_lib::recognition::MediaParser,
    nav: components::Nav,
    page: components::Page,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // let settings = if let Ok(s) = settings::Settings::load() {
        //     s
        // } else {
        //     settings::Settings::default()
        // };

        // let (regex_map, regex_sets) = if let Ok(r) = settings.recognition.regex_data() {
        //     println!("successfully loaded regexes");
        //     r
        // } else {
        //     (HashMap::new(), HashMap::new())
        // };

        let app = App {
            // settings: settings,
            media: "None Found".to_string(),
            // parser: fubuki_lib::recognition::MediaParser::new(regex_sets, regex_map),
            nav: components::Nav::new(),
            page: components::Page::default(),
        };

        if let Some(token) = settings::get_settings().write().unwrap().anilist.token() {
            println!("already authorized");
            // (app, Command::none())
            (
                app,
                Command::perform(
                    anilist::query_user(Some(token.clone())),
                    |result| match result {
                        Ok(user) => {
                            println!("got response\n{:#?}", user);
                            Message::AuthFailed
                        }
                        Err(err) => {
                            eprintln!("user query failed: {}", err);
                            Message::AuthFailed
                        }
                    },
                ),
            )
        } else {
            (
                app,
                Command::perform(anilist::auth(), |result| match result {
                    Ok(token) => Message::Authorized(token),
                    Err(err) => {
                        println!("authorization failed: {}", err);
                        Message::AuthFailed
                    }
                }),
            )
        }
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
            }
            Message::AuthFailed => {}
            Message::SearchMedia => {
                return Command::perform(MediaParser::detect_media(), Message::SearchResult);
            }
            Message::SearchResult(result) => {
                if let Some(title) = result {
                    self.update(Message::MediaFound(title));
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
