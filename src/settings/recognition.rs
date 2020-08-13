use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{default::Default, fs::File, io::BufReader, path::Path};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RecognitionData {
    pub anime: Vec<String>,
    pub manga: Vec<String>,
}

impl<'a> RecognitionData {
    const PATH: &'static str = "./res/recognition.json";
    const CUSTOM: &'static str = "./res/recognition_custom.json";

    pub fn load() -> Result<Self> {
        let path = Path::new(Self::PATH);
        match File::open(&path) {
            Ok(file) => {
                let rdr = BufReader::new(file);
                let r: RecognitionData = serde_json::from_reader(rdr)?;
                Ok(r)
            }
            Err(err) => {
                warn!("could not open recognition file {}: {}", Self::PATH, err);
                let default = Self::default();
                // default.save()?;
                Ok(default)
            }
        }
    }

    pub fn load_with_custom() -> Result<Self> {
        let mut r = Self::load()?;
        let path = Path::new(Self::CUSTOM);
        match File::open(&path) {
            Ok(file) => {
                let rdr = BufReader::new(file);
                let custom = serde_json::from_reader::<BufReader<File>, RecognitionData>(rdr);
                match custom {
                    Ok(custom) => {
                        r.anime = itertools::chain(r.anime, custom.anime).collect();
                        r.manga = itertools::chain(r.manga, custom.manga).collect();
                    }
                    Err(err) => {
                        warn!("error deserializing custom recognition data: {}", err);
                    }
                }
            }
            Err(err) => {
                warn!(
                    "could not open custom recognition file {}: {}",
                    Self::CUSTOM,
                    err
                );
            }
        }
        Ok(r)
    }

    // pub fn save(&self) -> Result<()> {
    //     let path = Path::new(Self::PATH);
    //     let file = OpenOptions::new()
    //         .write(true)
    //         .truncate(true)
    //         .open(path)?;
    //     let writer = BufWriter::new(file);
    //     serde_json::to_writer_pretty(writer, &self)?;
    //     Ok(())
    // }
}

// impl Default for RecognitionData {
//     fn default() -> Self {
//         RecognitionData {
//             anime: vec![
//                 "^(?P<title>.+) Episode (?P<episode>\\d+),.+?- Watch on Crunchyroll".to_string(),
//             ],
//             manga: vec![
//                 "^(?P<title>.+) - (Vol[.] (?P<volume>\\d+) )?(Ch[.] (?P<chapter>(\\d+[.])?\\d+) )?(?P<oneshot>Oneshot)?.*?- MangaDex".to_string(),
//             ],
//         }
//     }
// }
