use serde::{self, Deserialize, Serialize};
use chrono::{offset::Local, NaiveDate};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub media_list_options: Option<MediaListOptions>,
    pub options: Option<UserOptions>,
    pub avatar: Option<UserAvatar>,
}

impl User {
    pub fn get_avatar_url(&self) -> Option<String> {
        match &self.avatar {
            Some(avatar) => {
                avatar.medium.clone()
            },
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserOptions {
    pub profile_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAvatar {
    pub medium: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListOptions {
    pub score_format: Option<ScoreFormat>,
    pub anime_list: Option<MediaListTypeOptions>,
    pub manga_list: Option<MediaListTypeOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScoreFormat {
    #[serde(rename = "POINT_100")]
    Point100,
    #[serde(rename = "POINT_10_DECIMAL")]
    Point10Decimal,
    #[serde(rename = "POINT_10")]
    Point10,
    #[serde(rename = "POINT_5")]
    Point5,
    #[serde(rename = "POINT_3")]
    Point3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListTypeOptions {
    pub custom_lists: Option<Vec<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListCollection {
    pub lists: Option<Vec<Option<MediaListGroup>>>,
}

impl MediaListCollection {
    pub fn search_for_title(&mut self, search: &str) -> Option<&mut MediaList> {
        let lists = self.lists.as_mut()?;
        let mut found_entries = Vec::new();
        for list_group in lists.iter_mut() {
            let (entry, sim) = Self::search_entries(list_group.as_mut(), search);
            if sim == 1 as f64 {
                return entry;
            } else if entry.is_some() {
                found_entries.push((entry, sim));
                // return entry;
            }
        }
        let mut found_entry = None;
        let mut max_sim = 0.0;
        for (entry, sim) in found_entries {
            if sim > max_sim {
                max_sim = sim;
                found_entry = entry;
            }
        }
        found_entry
    }

    fn search_entries<'a>(list_group: Option<&'a mut MediaListGroup>, search: &str) -> (Option<&'a mut MediaList>, f64) {
        let entries = match list_group.and_then(|list_group| list_group.entries.as_mut()) {
            Some(entries) => entries,
            None => return (None, 0.0),
        };
        
        let mut found_entries = Vec::new();
        for entry in entries {
            let (entry, sim) = Self::entry_matching_search(entry, search);
            if sim == 1 as f64 {
                return (entry, sim);
            } else if entry.is_some() {
                found_entries.push((entry, sim));
                // return entry;
            }
        }
        let mut found_entry = None;
        let mut max_sim = 0.0;
        for (entry, sim) in found_entries {
            if sim > max_sim {
                max_sim = sim;
                found_entry = entry;
            }
        }
        (found_entry, max_sim)
    }
    
    fn entry_matching_search<'a>(entry: &'a mut Option<MediaList>, search: &str) -> (Option<&'a mut MediaList>, f64) {
        let entry = entry.as_mut();
        if let Some(entry) = entry {
            let media = entry.media.as_ref();
            if let Some(media) = media {
                for title in media.all_titles() {
                    let sim = strsim::normalized_levenshtein(title, search);
                    // println!("    similarity of {} between {} and {}", sim, search, title);
                    if sim >= 0.85 {
                        return (Some(entry), sim);
                    }
                }
            }
        }
        (None, 0.0)
    }
}

impl Media {
    pub fn all_titles(&self) -> Vec<&String> {
        let mut titles = Vec::new();
        if let Some(title) = &self.title {
            let t: [Option<&String>; 4] = [title.romaji.as_ref(), title.user_preferred.as_ref(), title.native.as_ref(), title.english.as_ref()];
            let t: Vec<&String> = t
                .iter()
                .filter_map(|item| *item)
                .collect();
            titles = itertools::chain(titles, t).collect();
        }
        
        if let Some(synonyms) = &self.synonyms {
            let syn: Vec<&String> = synonyms
                .iter()
                .filter_map(|title| title.as_ref())
                .collect();
            titles = itertools::chain(titles, syn).collect();
        }
        titles
    }

    pub fn preferred_title(&self) -> Option<String> {
        self.title.as_ref()?.user_preferred.clone()
    }

    pub fn cover_image_url(&self) -> Option<String> {
        self.cover_image.as_ref()?.large.clone()
    }

    pub fn description(&mut self) -> Option<String> {
        use once_cell::sync::Lazy;
        static HTML_REGEX: Lazy<Result<regex::Regex, regex::Error>> = Lazy::new(|| regex::Regex::new("<.+?>"));
        if let Some(desc) = &mut self.description {
            *desc = desc.replace("<br>", "\n");
            *desc = desc.replace("<br/>", "\n");
            *desc = desc.replace("<br />", "\n");
            match HTML_REGEX.as_ref() {
                Ok(html_regex) => {
                    *desc = html_regex.replace_all(desc, "").into();
                },
                _ => {},
            }
            self.description.clone()
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListGroup {
    pub entries: Option<Vec<Option<MediaList>>>,
    pub name: Option<String>,
    pub is_custom_list: Option<bool>,
    pub is_split_completed_list: Option<bool>,
    pub status: Option<MediaListStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaListStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaList {
    pub id: i32,
    pub media_id: i32,
    pub status: Option<MediaListStatus>,
    pub progress: Option<i32>,
    pub progress_volumes: Option<i32>,
    // pub score: Option<f64>
    // repeat, priority, private, notes, hiddenFromStatusLists, customLists
    // startedAt, completedAt,
    pub started_at: Option<FuzzyDate>,
    pub completed_at: Option<FuzzyDate>,
    pub media: Option<Media>,
}

impl MediaList {
    pub fn update_progress(&mut self, progress: Option<f64>, progress_volumes: Option<f64>) -> bool {
        let media = if let Some(media) = &mut self.media { media } else { return false };
        let media_type = if let Some(media_type) = &media.media_type { media_type } else { return false };
        let mut updated = false;
        match media_type {
            MediaType::Anime => {
                let episodes = match self.progress {
                    Some(episodes) => {
                        if let Some(progress) = progress {
                            if progress as i32 > episodes {
                                updated = true;
                                progress as i32
                            } else {
                                episodes
                            }
                        } else {
                            episodes
                        }
                    },
                    None => progress.unwrap_or_default() as i32,
                };
                self.progress = Some(episodes);
            },
            MediaType::Manga => {
                let chapters = match self.progress {
                    Some(chapters) => {
                        if let Some(progress) = progress {
                            if progress as i32 > chapters {
                                updated = true;
                                progress as i32
                            } else {
                                chapters
                            }
                        } else {
                            chapters
                        }
                    },
                    None => progress.unwrap_or_default() as i32,
                };

                let volumes = match self.progress_volumes {
                    Some(volumes) => {
                        if let Some(progress_volumes) = progress_volumes {
                            if progress_volumes as i32 - 1 > volumes {
                                updated = true;
                                progress_volumes as i32 - 1
                            } else {
                                volumes
                            }
                        } else {
                            volumes
                        }
                    },
                    None => progress_volumes.unwrap_or_default() as i32,
                };

                self.progress = Some(chapters);
                self.progress_volumes = Some(volumes);
            },
        }

        if updated && self.progress.is_some(){
            let progress = self.progress.unwrap();
            if progress == 1 && self.status.is_some() {
                if let Some(status) = &mut self.status {
                    match status {
                        MediaListStatus::Current | MediaListStatus::Planning | MediaListStatus::Dropped | MediaListStatus::Paused => {
                            *status = MediaListStatus::Current;
                            self.started_at = Some(FuzzyDate::today_local());
                        },
                        _ => {},
                    }
                }
            }
            if let Some(media) = &mut self.media {
                if let Some(media_type) = &media.media_type {
                    match media_type {
                        MediaType::Anime => {
                            if let Some(episodes) = media.episodes {
                                if progress == episodes {
                                    self.status = Some(MediaListStatus::Completed);
                                    self.completed_at = Some(FuzzyDate::today_local());
                                }
                            }
                        },
                        MediaType::Manga => {
                            if let Some(chapters) = media.chapters {
                                if progress == chapters {
                                    self.status = Some(MediaListStatus::Completed);
                                    self.completed_at = Some(FuzzyDate::today_local());
                                }
                            }
                            if let Some(volumes) = media.volumes {
                                if progress == volumes {
                                    self.status = Some(MediaListStatus::Completed);
                                    self.completed_at = Some(FuzzyDate::today_local());
                                }
                            }
                        },
                    }
                }
            }
        }

        updated
    }

    // TODO: Check media format (doujin, movie, etc) when making this string
    // pub fn current_media_string(&self) -> String {
    //     match &self.media {
    //         Some(media) => match &media.media_type {
    //             Some(media_type) => match media_type {
    //                 MediaType::Anime => {
    //                     if let Some(p) = self.progress {
    //                         return format!("Watching Episode {}", p);
    //                     }
    //                     return String::default();
    //                 },
    //                 MediaType::Manga => {
    //                     let mut s = String::default();
    //                     if let Some(p) = self.progress_volumes {
    //                         s = format!("Reading Vol. {}", p+1);
    //                     }
    //                     if let Some(p) = self.progress {
    //                         s = format!("{}, Ch. {}", s, p);
    //                     }
    //                     return s;
    //                 },
    //             },
    //             None => String::default(),
    //         },
    //         None => String::default(),
    //     }
    // }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

impl FuzzyDate {
    pub fn today_local() -> Self {
        let date = Local::today().naive_local();

        let year = Self::from_format(&date, "%Y");
        let month = Self::from_format(&date, "%m");
        let day = Self::from_format(&date, "%d");

        FuzzyDate { year, month, day }
    }

    fn from_format(date: &NaiveDate, fmt: &str) -> Option<i32> {
        match date.format(fmt).to_string().parse::<i32>() {
            Ok(x) => Some(x),
            Err(err) => {
                println!("date int parse error: {}", err);
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: i32,
    pub title: Option<MediaTitle>,
    #[serde(rename = "type")]
    pub media_type: Option<MediaType>,
    pub synonyms: Option<Vec<Option<String>>>,
    pub cover_image: Option<MediaCoverImage>,
    pub description: Option<String>,
    // ...
    pub episodes: Option<i32>,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaType {
    Anime,
    Manga,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaCoverImage {
    large: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_type() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            //#[serde(alias = "type")]
            #[serde(rename(serialize = "type", deserialize = "type"))]
            media_type: Option<MediaType>,
        }

        let j = r#"{"type":"MANGA"}"#;

        let expected = Data {
            media_type: Some(MediaType::Manga),
        };

        match serde_json::from_str::<Data>(j) {
            Ok(actual) => assert_eq!(expected.media_type, actual.media_type),
            Err(err) => panic!(err),
        }

        match serde_json::to_string(&expected) {
            Ok(serialized) => {
                assert_eq!(j, &serialized);
            }
            Err(err) => panic!(err),
        }
    }
}
