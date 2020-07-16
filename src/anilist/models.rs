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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub name: String,
    pub options: Option<UserOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaListCollection {
    pub lists: Option<Vec<Option<MediaListGroup>>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaListGroup {
    pub entries: Option<Vec<Option<MediaList>>>,
    pub name: Option<String>,
    pub is_custom_list: Option<bool>,
    pub is_split_completed_list: Option<bool>,
    pub status: Option<MediaListStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaListStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: i32,
    pub title: Option<MediaTitle>,
    #[serde(rename = "type")]
    pub media_type: Option<MediaType>,
    pub synonyms: Option<Vec<Option<String>>>,
    // ...
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
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
