#[cfg(windows)]
extern crate winapi;

pub mod anilist;
pub mod app;
pub mod recognition;
pub mod settings;
pub mod ui;

use anyhow::Result;
use app::App;
use iced::{Application, Settings};

//#![windows_subsystem = "windows"] // Tells windows compiler not to show console window

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

fn initialize_logger() -> Result<()> {
    let stdout = ConsoleAppender::builder().build();
    let path = crate::settings::file_path("fubuki.log")?;
    let app_dir_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("app_dir", Box::new(app_dir_appender)))
        .logger(
            Logger::builder()
                .appender("app_dir")
                .additive(false)
                .build("app", LevelFilter::Info),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))?;

    log4rs::init_config(config)?;

    Ok(())
}

fn main() -> Result<()> {
    initialize_logger()?;
    App::run(Settings::default());
    Ok(())
}
