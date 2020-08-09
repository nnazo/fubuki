mod anilist;
mod recognition;
pub mod settings;

pub use anilist::AniListData;
pub use recognition::RecognitionData;
pub use settings::{Settings, SETTINGS};

use anyhow::Result;
use app_dirs2::*;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::RwLock;

pub fn get_settings() -> &'static Lazy<RwLock<settings::Settings>> {
    &SETTINGS
}

const FUBUKI: AppInfo = AppInfo {
    name: "Fubuki",
    author: "nnazo",
};

fn file_path(path: &str) -> Result<PathBuf> {
    let mut p = app_root(AppDataType::UserData, &FUBUKI)?;
    p.push(path);
    Ok(p)
}
