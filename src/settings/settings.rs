use super::file_path;
use super::{AniListData, RecognitionData};
use anyhow::Result;
use log::warn;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    sync::RwLock,
};

pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| match Settings::load() {
    Ok(settings) => RwLock::new(settings),
    Err(err) => {
        warn!("settings load err: {}", err);
        RwLock::new(Settings::default())
    }
});
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(skip)]
    pub anilist: AniListData,
    #[serde(skip)]
    pub recognition: RecognitionData,
    pub update_delay: u64,
}

impl Settings {
    const FILE: &'static str = "general_settings.json";

    pub fn load() -> Result<Self> {
        let path = file_path(Self::FILE)?;
        let settings: Result<Settings> = match File::open(&path) {
            Ok(file) => {
                let rdr = BufReader::new(file);
                Ok(serde_json::from_reader(rdr)?)
            }
            Err(err) => {
                warn!("could not open settings file {:?}: {}", path, err);
                let default = Self::default();
                default.save()?;
                Ok(default)
            }
        };
        let settings = match settings {
            Ok(settings) => settings,
            Err(_) => Self::default(),
        };
        Ok(Settings {
            anilist: AniListData::load()?,
            recognition: RecognitionData::load_with_custom()?,
            ..settings
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = file_path(Self::FILE)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;

        // self.anilist.save()?;
        // self.recognition.save()?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            anilist: AniListData::default(),
            recognition: RecognitionData::default(),
            update_delay: 5,
        }
    }
}
