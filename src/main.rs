#[cfg(windows)]
extern crate winapi;

pub mod anilist;
pub mod app;
pub mod recognition;
pub mod resources;
pub mod settings;
pub mod ui;

use anyhow::Result;
use app::App;
use iced::{Application, Settings};
use settings::file_path;

//#![windows_subsystem = "windows"] // Tells windows compiler not to show console window

use log::{warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::json::JsonEncoder;

fn initialize_logger() -> Result<()> {
    let stdout = ConsoleAppender::builder().build();
    let path = file_path("fubuki.log")?;
    {
        // Truncate the log file if it exists
        match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path.clone())
        {
            _ => {}
        }
    }
    let app_dir_appender = FileAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build(path)?;

    let mut logger = Logger::builder();
    logger = logger.appender("app_dir");

    // Remove logs from stdout in release mode
    #[cfg(not(debug_assertions))]
    {
        logger = logger.additive(false);
    }

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("app_dir", Box::new(app_dir_appender)))
        .logger(logger.build("fubuki", LevelFilter::Debug))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))?;

    log4rs::init_config(config)?;

    Ok(())
}

fn main() -> Result<()> {
    initialize_logger()?;
    let mut settings = Settings::default();
    if let Err(err) = app::set_icon(&mut settings) {
        warn!("could not load application icon: {}", err);
    };
    app::set_min_dimensions(&mut settings);
    App::run(settings);
    Ok(())
}
