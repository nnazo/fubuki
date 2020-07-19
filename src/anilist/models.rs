use serde::{self, Deserialize, Serialize};
// use std::default::Default;

// pub trait Media {
//     fn length() -> i32; // wait what aobut decimal chapters for manga fuck
//     fn duration() -> i32;
// }

// may also wanna move this into like a "models" module where i can have User as well and anything else that comes up

// #[derive(Clone, Default, Debug)]  // maybe can just read this in as a string ... ? and other things too... dunno if i should
// pub enum MediaFormat {
//     Tv,
//     TvShort,
//     Movie,
//     Special,
//     Ova,
//     Ona,
//     Music,
//     Manga,
//     Novel,
//     Oneshot,
// }

// #[derive(Clone, Default, Debug)]
// pub struct FuzzyDate {
//     pub year: i32,
//     pub month: i32,
//     pub day: i32,
// }

// #[derive(Clone, Default, Debug)]
// pub struct Title {
//     pub romaji: String,
//     pub english: String,
//     pub native: String,
// }

// #[derive(Clone, Default, Debug)]
// pub struct Anime {
//     pub id: i32,
//     pub title: Title,
//     // pub format: MediaFormat,
//     pub description: String,
//     pub start_date: FuzzyDate,
//     pub end_date: FuzzyDate,
//     // pub season:
// }

// #[derive(Clone, Default, Debug)]
// pub struct Manga {

// }

// impl Media for Anime {

// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub options: Option<UserOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserOptions {
    pub profile_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaListOptions {
    pub score_format: Option<ScoreFormat>,
    pub anime_list: Option<MediaListTypeOptions>,
    pub manga_list: Option<MediaListTypeOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScoreFormat {
    Point100,
    Point10Decimal,
    Point10,
    Point5,
    Point3,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn search_title(&mut self, search: &str) -> Option<&mut MediaList> {
        if let Some(lists) = &mut self.lists {
            for list_group in lists.iter_mut() {
                if let Some(list_group) = list_group {
                    // println!("checking list group {:?}", list_group.name);
                    if let Some(entries) = &mut list_group.entries {
                        for entry in entries.iter_mut() {
                            if let Some(entry) = entry {
                                if let Some(media) = &entry.media {
                                    // println!("  checking media id {:?}", media.id);
                                    let titles = media.all_titles();
                                    for title in titles {
                                        let sim = strsim::normalized_levenshtein(title, search);
                                        // println!("    similarity of {} between {} and {}", sim, search, title);
                                        if sim >= 0.85 {
                                            return Some(entry);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl Media {
    pub fn all_titles(&self) -> Vec<&String> {
        let mut titles = Vec::new();
        if let Some(title) = &self.title {
            Self::add_title(&mut titles, &title.romaji);
            Self::add_title(&mut titles, &title.user_preferred);
            Self::add_title(&mut titles, &title.native);
            Self::add_title(&mut titles, &title.english);
        }
        if let Some(synonyms) = &self.synonyms {
            synonyms
                .iter()
                .for_each(|title| Self::add_title(&mut titles, title));
        }
        titles
    }

    fn add_title<'a>(v: &mut Vec<&'a String>, title: &'a Option<String>) {
        if let Some(title) = title {
            v.push(title);
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
                        if let Some(progress_volumes) = progress {
                            if progress_volumes as i32 > volumes {
                                updated = true;
                                progress_volumes as i32
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
        updated
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
    // ...
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
