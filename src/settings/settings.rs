use super::{
    AniListData,
    RecognitionData,
};
use std::{
    default::Default,
    sync::RwLock,
};
use anyhow::Result;
use once_cell::sync::Lazy;

pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    match Settings::load() {
        Ok(settings) => RwLock::new(settings),
        Err(err) => {
            eprintln!("settings load err: {}", err);
            RwLock::new(Settings::default())
        },
    }
});
#[derive(Debug, Default)]
pub struct Settings {
    pub anilist: AniListData,
    pub recognition: RecognitionData,
}

impl Settings {
    pub fn load() -> Result<Self> {
        Ok(Settings {
            anilist: AniListData::load()?,
            recognition: RecognitionData::load_with_custom()?,
        })
    }
    
    pub fn save(&self) -> Result<()> {
        self.anilist.save()?;
        // self.recognition.save()?;
        Ok(())
    }
}

// pub fn settings() -> Settings {
//     SETTINGS.lock().
// }