use std::{
    fs::{File, OpenOptions}, 
    path::PathBuf, 
    io::{BufWriter, BufReader}, 
    default::Default
};
use app_dirs2::*;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct AniListData {
    token: Option<String>,
}

impl AniListData {
    const FUBUKI: AppInfo = AppInfo{name: "Fubuki", author: "nnazo"};
    const FILE: &'static str = "anilist_data.json";

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;
        match File::open(&path) {
            Ok(file) => {
                let rdr = BufReader::new(file);
                Ok(serde_json::from_reader(rdr)?)
            },
            Err(err) => {
                println!("could not open {:?}: {}", path, err);
                let default = Self::default();
                default.save()?;
                Ok(default)
            }
        }
    }

    pub fn save_token(&mut self, tok: &str) {
        self.token = Some(tok.to_string())
    }

    pub fn forget_token(&mut self) -> Result<()> {
        self.token = None;
        self.save()?;
        Ok(())
    }

    pub fn token(&self) -> &Option<String> {
        &self.token
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    fn file_path() -> Result<PathBuf> {
        let mut path = app_root(AppDataType::UserData, &Self::FUBUKI)?;
        path.push(Self::FILE);
        Ok(path)
    }
}

impl Default for AniListData {
    fn default() -> Self {
        AniListData {
            token: None,
        }
    }
}