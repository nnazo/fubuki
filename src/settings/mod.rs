mod settings;
mod anilist;
mod recognition;

pub use settings::{Settings, SETTINGS};
pub use anilist::AniListData;
pub use recognition::RecognitionData;

use once_cell::sync::Lazy;
use std::sync::RwLock;

pub fn get_settings() -> &'static Lazy<RwLock<settings::Settings>> {
    &SETTINGS
}