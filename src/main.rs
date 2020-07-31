// #[macro_use] extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
#[macro_use]
extern crate lazy_static;

pub mod anilist;
pub mod app;
pub mod recognition;
pub mod settings;
pub mod ui;

use app::App;
use iced::{Application, Settings};

//#![windows_subsystem = "windows"] // Tells windows compiler not to show console window

fn main() {
    App::run(Settings::default());
}
