use serde_json::{map::Map, value::Value::{self, Object}};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use regex::{Regex, RegexSet};
use std::collections::HashMap;

pub enum ParseError {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotAMap(),
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> ParseError {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => ParseError::Io(err.into()),
            Category::Syntax | Category::Data | Category::Eof => ParseError::Json(err),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError::Io(err.into())
    }
}

pub fn init_json_map() -> Result<Map<String, Value>, ParseError> {
    let path = Path::new("./res/recognition.json");
    let file = File::open(path)?;
    
    let reader = BufReader::new(file);
    let value = serde_json::from_reader(reader)?;

    match value {
        Object(map) => Ok(map),
        _ => Err(ParseError::NotAMap()),
    }
}

pub fn init_regex_maps(json_map: Map<String, Value>) -> Result<(HashMap<String, RegexSet>, HashMap<String, Regex>), Box<dyn std::error::Error>> {
    let mut regex_set_map = HashMap::new();
    let mut regex_map = HashMap::new();

    for (key, value) in json_map {
        if let Some(array) = value.as_array() {
            let mut regexes = Vec::new();
            for regex in array.iter() {
                if let Value::String(str) = regex {
                    let regex = Regex::new(str)?;
                    regex_map.insert(String::from(str), regex);
                    regexes.push(str);
                }
            }
            match RegexSet::new(regexes) {
                Ok(regex_set) => {
                    regex_set_map.insert(key, regex_set);
                },
                Err(err) => {
                    return Err(Box::new(err));
                },
            }
        }        
    }

    Ok((regex_set_map, regex_map))
}
