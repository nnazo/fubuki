use super::get_window_titles;
use crate::settings;
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::{Captures, Regex, RegexSet};
use std::collections::HashMap;

static MEDIA_PARSER: Lazy<MediaParser> = Lazy::new(|| {
    let settings = settings::get_settings().read().unwrap();
    MediaParser::new(&settings.recognition.anime, &settings.recognition.manga).unwrap_or_default()
});

#[derive(Default, Clone, Debug)]
pub struct MediaParser {
    anime: Option<RegexSet>,
    manga: Option<RegexSet>,
    cache: HashMap<String, Regex>,
}

impl MediaParser {
    pub fn new(anime_regexes: &Vec<String>, manga_regexes: &Vec<String>) -> Result<Self> {
        let mut cache = HashMap::new();
        for pattern in anime_regexes.iter() {
            let regex = Regex::new(pattern)?;
            cache.insert(pattern.clone(), regex);
        }
        for pattern in manga_regexes.iter() {
            let regex = Regex::new(pattern)?;
            cache.insert(pattern.clone(), regex);
        }

        Ok(MediaParser {
            anime: Some(RegexSet::new(anime_regexes)?),
            manga: Some(RegexSet::new(manga_regexes)?),
            cache,
        })
    }

    pub fn match_set<'a>(&'a self, regex_set: &'a RegexSet, window_title: &str) -> Option<&'a str> {
        let mut matches = regex_set.matches(window_title).into_iter();
        let match_ndx = matches.next()?;
        Some(&regex_set.patterns()[match_ndx])
    }

    pub fn parse_anime<'a>(&self, window_title: &'a str) -> Option<Captures<'a>> {
        let set = self.anime.as_ref()?;
        self.parse_media(set, window_title)
    }

    pub fn parse_manga<'a>(&self, window_title: &'a str) -> Option<Captures<'a>> {
        let set = self.manga.as_ref()?;
        self.parse_media(set, window_title)
    }

    pub fn parse_media<'a>(
        &self,
        regex_set: &RegexSet,
        window_title: &'a str,
    ) -> Option<Captures<'a>> {
        let pattern = self.match_set(regex_set, window_title)?;
        if let Some(regex) = self.cache.get(pattern) {
            regex.captures(window_title)
        } else {
            None
        }
    }

    pub async fn detect_media() -> Option<String> {
        for title in get_window_titles() {
            if let Some(anime_captures) = MEDIA_PARSER.parse_anime(&title) {
                return Some(String::from(anime_captures.name("title")?.as_str()));
            } else if let Some(manga_captures) = MEDIA_PARSER.parse_manga(&title) {
                return Some(String::from(manga_captures.name("title")?.as_str()));
            }
        }

        None
    }
}
