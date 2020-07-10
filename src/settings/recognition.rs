use std::{
    fs::{File, OpenOptions},
    path::Path,
    io::{BufWriter, BufReader},
    default::Default,
    collections::HashMap,
};
use serde::{Serialize, Deserialize};
use regex::{Regex, RegexSet};
use anyhow::Result;
use fubuki_lib::recognition::MediaParser;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RecognitionData {
    pub player: Vec<String>,
    pub browser: Vec<String>,
    pub anime: Vec<String>,
    pub manga: Vec<String>,

    #[serde(skip_serializing, skip_deserializing)]
    pub parser: MediaParser,
}

impl<'a> RecognitionData {
    const PATH: &'static str = "./res/recognition.json";
    const CUSTOM: &'static str = "./res/recognition_custom.json";

    pub fn load() -> Result<Self> {
        let path = Path::new(Self::PATH);
        match File::open(&path) {
            Ok(file) => {
                let rdr = BufReader::new(file);
                let mut r: RecognitionData = serde_json::from_reader(rdr)?;
                let (regex_map, regex_sets) = r.regex_data()?;
                r.parser = MediaParser::new(regex_sets, regex_map);
                Ok(r)
            },
            Err(err) => {
                println!("could not open {}: {}", Self::PATH, err);
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
                        for rgx in custom.player {
                            r.player.push(rgx);
                        }
                        for rgx in custom.browser {
                            r.browser.push(rgx);
                        }
                        for rgx in custom.anime {
                            r.anime.push(rgx);
                        }
                        for rgx in custom.manga {
                            r.manga.push(rgx);
                        }
                    },
                    Err(err) => {
                        println!("error deserializing custom recognition data: {}", err);
                    },
                }
            },
            Err(err) => {
                println!("could not open custom {}: {}", Self::CUSTOM, err);
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

    pub fn regex_data(&self) -> Result<(HashMap<String, Regex>, HashMap<String, RegexSet>)> {
        let mut map = HashMap::new();
        map.insert("player", &self.player);
        map.insert("browser", &self.browser);
        map.insert("anime", &self.manga);
        map.insert("manga", &self.manga);

        let mut regex_map = HashMap::new();
        let mut regex_set_map = HashMap::new();

        for (rgx, vec) in map {
            let mut regexes = Vec::new();
            for val in vec {
                let regex = Regex::new(val)?;
                regex_map.insert(String::from(rgx), regex);
                regexes.push(val);
            }
            let regex_set = RegexSet::new(regexes)?;
            regex_set_map.insert(String::from(rgx), regex_set);
        }

        return Ok((regex_map, regex_set_map))
    }
}

// impl Default for RecognitionData {
//     fn default() -> Self {
//         RecognitionData {
//             player: vec![
//                 "(?P<tab>.+) - VLC media player".to_string(),
//             ],
//             browser: vec![
//                 "(?P<tab>.+) - Mozilla Firefox".to_string(),
//                 "(?P<tab>.+) - Google Chrome".to_string(),
//             ],
//             anime: vec![
//                 "(?P<title>.+) Episode (?P<episode>\\d+),.+?- Watch on Crunchyroll".to_string(),
//             ],
//             manga: vec![
//                 "(?P<title>.+) - (Vol[.] (?P<volume>\\d+) )?(Ch[.] (?P<chapter>(\\d+[.])?\\d+) )?(?P<oneshot>Oneshot)?.*?- MangaDex".to_string(),
//             ],
//         }
//     }
// }