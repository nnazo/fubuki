use chrono::{offset::Local, NaiveDate};
use serde::{self, Deserialize, Serialize};

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
            Some(avatar) => avatar.medium.clone(),
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
    pub fn compute_progress_offset_for_sequel(
        &self,
        id: i32,
        new_progress: i32,
    ) -> Option<(i32, i32)> {
        let media = self.find_entry_by_id(id);
        match media {
            Some(entry) => match &entry.media {
                Some(media) => {
                    let sequel = media.find_anime_sequel();
                    if let Some(sequel) = sequel {
                        let offset_progress =
                            self.compute_progress_offset_by_id(sequel.id, new_progress as i32);
                        if let Some(offset_progress) = offset_progress {
                            Some((offset_progress, sequel.id))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                None => None,
            },
            None => None,
        }
    }

    pub fn compute_progress_offset_by_id(&self, id: i32, new_progress: i32) -> Option<i32> {
        let entry = self.find_entry_by_id(id);
        match entry {
            Some(entry) => self.compute_progress_offset(entry, new_progress),
            None => None,
        }
    }

    pub fn compute_progress_offset(&self, entry: &MediaList, new_progress: i32) -> Option<i32> {
        let media = entry.media.as_ref()?;
        let length = media.episodes.unwrap_or_default();
        if new_progress > length {
            match self.compute_total_episodes(media, new_progress) {
                Some(total) => Some(new_progress - total + length),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn compute_total_episodes(&self, media: &Media, new_progress: i32) -> Option<i32> {
        let relations = media.relations.as_ref()?;
        let edges: Vec<&MediaEdge> = relations
            .edges
            .as_ref()?
            .iter()
            .filter_map(|edge| edge.as_ref())
            .filter(|edge| match edge.relation_type {
                Some(MediaRelation::Prequel) => true,
                _ => false,
            })
            .filter(|edge| match &edge.node {
                Some(node) => match &node.format {
                    Some(MediaFormat::Tv) => true,
                    _ => false,
                },
                _ => false,
            })
            .collect();

        let length = media.episodes?;

        if length < new_progress {
            if edges.len() == 1 {
                let edge = edges[0];
                let prequel = self.find_entry_by_id(edge.node.as_ref()?.id);
                let prequel_media = prequel.as_ref()?.media.as_ref()?;
                match prequel_media.episodes {
                    Some(episodes) => {
                        if episodes + length < new_progress {
                            let sub_offset =
                                self.compute_total_episodes(prequel_media, new_progress);
                            match sub_offset {
                                Some(sub_offset) => Some(length + sub_offset),
                                None => Some(length),
                            }
                        } else {
                            Some(episodes + length)
                        }
                    }
                    None => None,
                }
            } else {
                Some(length)
            }
        } else {
            Some(length)
        }
    }

    pub fn find_entry_by_id(&self, id: i32) -> Option<&MediaList> {
        let lists = self.lists.as_ref()?;
        let lists: Vec<&MediaListGroup> = lists
            .iter()
            .filter_map(|list_group| list_group.as_ref())
            .collect();
        for list in lists {
            let entries: Vec<&MediaList> = match &list.entries {
                Some(entries) => entries,
                None => continue,
            }
            .iter()
            .filter_map(|entry| entry.as_ref())
            .collect();

            for entry in entries {
                if entry.media_id == id {
                    return Some(entry);
                }
            }
        }

        None
    }

    pub fn find_entry_by_id_mut(&mut self, id: i32) -> Option<&mut MediaList> {
        let lists = self.lists.as_mut()?;
        let lists: Vec<&mut MediaListGroup> = lists
            .iter_mut()
            .filter_map(|list_group| list_group.as_mut())
            .collect();
        for list in lists {
            let entries: Vec<&mut MediaList> = match &mut list.entries {
                Some(entries) => entries,
                None => continue,
            }
            .iter_mut()
            .filter_map(|entry| entry.as_mut())
            .collect();

            for entry in entries {
                if entry.media_id == id {
                    return Some(entry);
                }
            }
        }

        None
    }

    pub fn collection_best_id_for_search(&mut self, search: &str, oneshot: bool) -> Option<i32> {
        let lists = self.lists.as_mut()?;
        let list_groups: Vec<&mut MediaListGroup> = lists
            .into_iter()
            .filter_map(|list_group| list_group.as_mut())
            .collect();
        let mut entries: Vec<Option<&MediaList>> = Vec::new();
        for list_group in list_groups {
            let e = match &mut list_group.entries {
                Some(entries) => entries,
                None => continue,
            };
            let mut e: Vec<Option<&MediaList>> = e
                .iter()
                .filter_map(|m| m.as_ref())
                .map(|m| Some(m))
                .collect();
            entries.append(&mut e);
        }

        let media: Vec<Option<&Media>> = entries
            .iter()
            .filter_map(|entry| *entry)
            .filter_map(|entry| entry.media.as_ref())
            .filter(|media| match &media.format {
                Some(fmt) => match fmt {
                    MediaFormat::Oneshot => oneshot,
                    _ => false,
                },
                None => false,
            })
            .map(|media| Some(media))
            .collect();

        Self::best_id_for_search(&media, search, oneshot)
    }

    pub fn best_id_for_search(
        media: &[Option<&Media>],
        search: &str,
        oneshot: bool,
    ) -> Option<i32> {
        let mut media_in_list: Vec<&Media> = media
            .into_iter()
            .filter_map(|media| *media)
            .filter_map(|media| match media.media_list_entry {
                Some(_) => Some(media),
                None => None,
            })
            .collect();

        // Look for oneshots only
        if oneshot {
            media_in_list = media_in_list
                .into_iter()
                .filter_map(|media| match media.format {
                    Some(MediaFormat::Oneshot) => Some(media),
                    _ => None,
                })
                .collect();
        } else {
            // filter out oneshots
            media_in_list = media_in_list
                .into_iter()
                .filter_map(|media| match media.format {
                    Some(MediaFormat::Oneshot) => None,
                    Some(_) => Some(media),
                    None => None,
                })
                .collect();
        }

        let mut best_is_licensed = false;
        let mut best_id = None;
        let mut best_sim = 0.0;
        for media in media_in_list {
            for title in media.all_titles() {
                let license = media.is_licensed.unwrap_or(false);
                let sim = strsim::normalized_levenshtein(title, search);
                // println!("    similarity of {} between {} and {}", sim, search, title);
                if sim >= 0.85 {
                    if sim > best_sim {
                        best_id = Some(media.id);
                        best_sim = sim;
                        best_is_licensed = license;
                    } else if sim == best_sim && !best_is_licensed {
                        // Prioritize licensed media over non-licensed media since a licensed version exists
                        best_id = Some(media.id);
                        best_sim = sim;
                        best_is_licensed = license;
                    }
                }
            }
        }

        best_id
    }

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

    fn search_entries<'a>(
        list_group: Option<&'a mut MediaListGroup>,
        search: &str,
    ) -> (Option<&'a mut MediaList>, f64) {
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

    fn entry_matching_search<'a>(
        entry: &'a mut Option<MediaList>,
        search: &str,
    ) -> (Option<&'a mut MediaList>, f64) {
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
            let t: [Option<&String>; 4] = [
                title.romaji.as_ref(),
                title.user_preferred.as_ref(),
                title.native.as_ref(),
                title.english.as_ref(),
            ];
            let t: Vec<&String> = t.iter().filter_map(|item| *item).collect();
            titles = itertools::chain(titles, t).collect();
        }

        if let Some(synonyms) = &self.synonyms {
            let syn: Vec<&String> = synonyms.iter().filter_map(|title| title.as_ref()).collect();
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
        static HTML_REGEX: Lazy<Result<regex::Regex, regex::Error>> =
            Lazy::new(|| regex::Regex::new("<.+?>"));
        if let Some(desc) = &mut self.description {
            *desc = desc.replace("<br>", "\n");
            *desc = desc.replace("<br/>", "\n");
            *desc = desc.replace("<br />", "\n");
            match HTML_REGEX.as_ref() {
                Ok(html_regex) => {
                    *desc = html_regex.replace_all(desc, "").into();
                }
                _ => {}
            }
            self.description.clone()
        } else {
            None
        }
    }

    pub fn find_anime_sequel(&self) -> Option<&Media> {
        let relations = self.relations.as_ref()?;
        let edges: Vec<&MediaEdge> = relations
            .edges
            .as_ref()?
            .iter()
            .filter_map(|edge| edge.as_ref())
            .filter(|edge| match edge.relation_type {
                Some(MediaRelation::Sequel) => true,
                _ => false,
            })
            .filter(|edge| match edge.node.as_ref() {
                Some(node) => match node.format {
                    Some(MediaFormat::Tv) => true,
                    _ => false,
                },
                None => false,
            })
            .collect();

        if edges.len() == 1 {
            edges[0].node.as_ref()
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
    pub score: Option<f64>,
    // repeat, priority, private, notes, hiddenFromStatusLists, customLists
    // startedAt, completedAt,
    pub started_at: Option<FuzzyDate>,
    pub completed_at: Option<FuzzyDate>,
    pub media: Option<Media>,
}

impl MediaList {
    pub fn update_progress(
        &mut self,
        progress: Option<f64>,
        progress_volumes: Option<f64>,
    ) -> bool {
        let media = if let Some(media) = &mut self.media {
            media
        } else {
            return false;
        };
        let media_type = if let Some(media_type) = &media.media_type {
            media_type
        } else {
            return false;
        };
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
                    }
                    None => progress.unwrap_or_default() as i32,
                };
                self.progress = Some(episodes);
            }
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
                    }
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
                    }
                    None => progress_volumes.unwrap_or_default() as i32,
                };

                self.progress = Some(chapters);
                self.progress_volumes = Some(volumes);
            }
        }

        if updated && self.progress.is_some() {
            let progress = self.progress.unwrap();
            if progress == 1 && self.status.is_some() {
                if let Some(status) = &mut self.status {
                    match status {
                        MediaListStatus::Current
                        | MediaListStatus::Planning
                        | MediaListStatus::Dropped
                        | MediaListStatus::Paused => {
                            *status = MediaListStatus::Current;
                            self.started_at = Some(FuzzyDate::today_local());
                        }
                        _ => {}
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
                        }
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
                        }
                    }
                }
            }
        }

        updated
    }

    pub fn progress_string(&self) -> String {
        match &self.media {
            Some(media) => match media.media_type {
                Some(MediaType::Anime) => Self::progress_string_util(self.progress, media.episodes),
                Some(MediaType::Manga) => Self::progress_string_util(self.progress, media.chapters),
                None => Self::progress_string_util(self.progress, None),
            },
            None => Self::progress_string_util(self.progress, None),
        }
    }

    pub fn progress_volumes_string(&self) -> String {
        match &self.media {
            Some(media) => Self::progress_string_util(self.progress_volumes, media.volumes),
            None => Self::progress_string_util(self.progress, None),
        }
    }

    fn progress_string_util(progress: Option<i32>, max: Option<i32>) -> String {
        let max = match max {
            Some(max) => format!("{}", max),
            None => "?".to_string(),
        };
        format!("{} / {}", progress.unwrap_or_default(), max)
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
    pub format: Option<MediaFormat>,
    pub is_licensed: Option<bool>,
    pub relations: Option<MediaConnection>,
    // ...
    pub episodes: Option<i32>,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
    pub media_list_entry: Option<Box<MediaList>>,
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

impl Default for MediaType {
    fn default() -> Self {
        MediaType::Anime
    }
}

impl MediaType {
    pub fn string(&self) -> &str {
        match self {
            MediaType::Anime => "Anime",
            MediaType::Manga => "Manga",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaCoverImage {
    large: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaConnection {
    pub edges: Option<Vec<Option<MediaEdge>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaEdge {
    pub node: Option<Media>,
    pub relation_type: Option<MediaRelation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaRelation {
    Adaptation,
    Prequel,
    Sequel,
    Parent,
    SideStory,
    Character,
    Summary,
    Alternative,
    SpinOff,
    Other,
    Source,
    Compilation,
    Contains,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    #[serde(rename = "ONE_SHOT")]
    Oneshot,
}

impl MediaFormat {
    pub fn str(&self) -> &str {
        match self {
            MediaFormat::Tv => "TV",
            MediaFormat::TvShort => "TV Short",
            MediaFormat::Movie => "Movie",
            MediaFormat::Special => "Special",
            MediaFormat::Ova => "OVA",
            MediaFormat::Ona => "ONA",
            MediaFormat::Music => "Music",
            MediaFormat::Manga => "Manga",
            MediaFormat::Novel => "Light Novel",
            MediaFormat::Oneshot => "Oneshot",
        }
    }
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
