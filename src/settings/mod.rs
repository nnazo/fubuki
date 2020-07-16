mod anilist;
mod recognition;
pub mod settings;

pub use anilist::AniListData;
pub use recognition::RecognitionData;
pub use settings::{Settings, SETTINGS};

use once_cell::sync::Lazy;
use std::sync::RwLock;

pub fn get_settings() -> &'static Lazy<RwLock<settings::Settings>> {
    &SETTINGS
}
