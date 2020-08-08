use anyhow::Result;
use super::file_path;
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct AniListData {
    token: Option<String>,
}

impl AniListData {
    const FILE: &'static str = "anilist_data.json";

    pub fn load() -> Result<Self> {
        let path = file_path(Self::FILE)?;
        match File::open(&path) {
            Ok(file) => {
                let rdr = BufReader::new(file);
                Ok(serde_json::from_reader(rdr)?)
            }
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
        let path = file_path(Self::FILE)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }
}

impl Default for AniListData {
    fn default() -> Self {
        AniListData { token: None }
    }
}
