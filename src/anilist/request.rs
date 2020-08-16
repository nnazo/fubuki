use super::models::{Media, MediaList, MediaListCollection, MediaType, User};
use crate::{resources::Resources, settings};
use anyhow::{anyhow, Result};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Map, Value};
// use std::path::Path;
use tokio::time;

#[derive(Deserialize, Debug)]
pub struct QueryError {
    pub message: Option<String>,
    pub status: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct QueryResponse<R> {
    pub data: Option<R>,
    pub errors: Option<Vec<QueryError>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ViewerResponse {
    pub viewer: Option<User>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaListCollectionResponse {
    pub media_list_collection: Option<MediaListCollection>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SaveMediaListEntryResponse {
    pub save_media_list_entry: Option<MediaList>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SearchResponse {
    pub page: SearchPage,
}

#[derive(Deserialize, Debug)]
pub struct SearchPage {
    pub media: Option<Vec<Option<Media>>>,
}

pub async fn query_graphql<R>(
    query_str: &str,
    variables: &Option<Map<String, Value>>,
    token: Option<String>,
) -> Result<QueryResponse<R>>
where
    R: DeserializeOwned,
{
    let query = if let Some(vars) = &variables {
        json!({ "query": query_str, "variables": vars })
    } else {
        json!({ "query": query_str })
    };

    let token = match token {
        Some(tok) => {
            if tok.is_empty() {
                return Err(anyhow!("Empty token string"));
            }
            tok
        }
        None => return Err(anyhow!("No token provided")),
    };

    let max_rate_limit_count = 5;
    for _ in 0..max_rate_limit_count {
        let client = Client::new();
        let resp = client
            .post("https://graphql.anilist.co")
            .header("ContentType", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&query)
            .send()
            .await?;

        match resp.status() {
            StatusCode::TOO_MANY_REQUESTS => {
                let secs;
                let retry = resp.headers().get("Retry-After");
                if let Some(val) = retry {
                    let header = String::from_utf8_lossy(val.as_bytes());
                    secs = header.parse::<u64>().unwrap_or(60);
                } else {
                    secs = 60;
                }
                time::delay_for(time::Duration::from_secs(secs)).await;
            }
            StatusCode::OK | _ => {
                let response: QueryResponse<R> = resp.json().await?;
                return Ok(response);
            }
        }
    }

    Err(anyhow!("Exceeded the maximum rate limit count (5)"))
}

pub async fn query_from_file<R>(
    path: &str,
    variables: &Option<Map<String, Value>>,
    token: Option<String>,
) -> Result<QueryResponse<R>>
where
    R: DeserializeOwned,
{
    let query: String = Resources::get(path).map_or_else(
        || Err(anyhow!("could not load query from \"{}\"", path)),
        |query| {
            std::str::from_utf8(&*query).map_or_else(
                |err| {
                    Err(anyhow!(
                        "failed to covert \"{}\" query to utf8: {}",
                        path,
                        err
                    ))
                },
                |s| Ok(s.to_string()),
            )
        },
    )?;
    query_graphql(&query, variables, token).await
}

pub async fn query_media_list(
    token: Option<String>,
    user_id: i32,
    media_type: MediaType,
) -> Result<QueryResponse<MediaListCollectionResponse>> {
    let variables = json!({
        "id": user_id,
        "type": media_type,
    });
    if let serde_json::Value::Object(variables) = variables {
        query_from_file("graphql/media_list.gql", &Some(variables), token).await
    } else {
        Err(anyhow!("media list query variables was not a json object"))
    }
}

pub async fn query_media_lists(
    token: Option<String>,
    user_id: i32,
) -> (
    Result<QueryResponse<MediaListCollectionResponse>>,
    Result<QueryResponse<MediaListCollectionResponse>>,
) {
    (
        query_media_list(token.clone(), user_id, MediaType::Anime).await,
        query_media_list(token, user_id, MediaType::Manga).await,
    )
}

pub async fn query_user(token: Option<String>) -> Result<QueryResponse<ViewerResponse>> {
    query_from_file("graphql/user.gql", &None, token).await
}

pub async fn update_media(
    token: Option<String>,
    media: MediaList,
) -> Result<QueryResponse<SaveMediaListEntryResponse>> {
    let variables = json!({
        "id": media.id,
        "status": media.status,
        "progress": media.progress,
        "progressVolumes": media.progress_volumes.unwrap_or_default(),
        "startedAt": media.started_at,
        "completedAt": media.completed_at,
    });
    if let serde_json::Value::Object(variables) = variables {
        query_from_file("graphql/update_media.gql", &Some(variables), token).await
    } else {
        Err(anyhow!("update media variables was not a json object"))
    }
}

pub async fn query_search(
    token: Option<String>,
    search: String,
    media_type: MediaType,
) -> Result<QueryResponse<SearchResponse>> {
    let variables = json!({
        "search": search,
        "mediaType": media_type,
    });
    if let serde_json::Value::Object(variables) = variables {
        query_from_file("graphql/search.gql", &Some(variables), token).await
    } else {
        Err(anyhow!("update media variables was not a json object"))
    }
}

use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Default)]
pub struct ListUpdateQueue {
    waiting: bool,
    requests: VecDeque<(MediaList, Instant)>,
}

impl ListUpdateQueue {
    pub fn enqueue(&mut self, media: MediaList) {
        let mut found = false;
        for (m, _) in self.requests.iter_mut() {
            if m.media_id == *&media.media_id {
                *m = media.clone();
                found = true;
                break;
            }
        }
        if !found {
            self.requests.push_back((media, Instant::now()));
        }
    }

    pub fn is_waiting(&self) -> bool {
        self.waiting
    }

    pub fn set_waiting(&mut self, waiting: bool) {
        self.waiting = waiting;
    }

    pub fn dequeue(&mut self) -> Option<MediaList> {
        match self.requests.front() {
            Some((_, earlier)) => {
                let update_delay = settings::get_settings().read().unwrap().update_delay;
                let elapsed = Instant::now().duration_since(*earlier);
                if elapsed.as_secs() >= update_delay && !self.waiting {
                    let front = self.requests.pop_front();
                    match front {
                        Some((media, _)) => Some(media),
                        None => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<MediaList> {
        match self.requests.remove(index) {
            Some((media, _)) => Some(media),
            None => None,
        }
    }

    pub fn find_index(&self, media_id: i32) -> Option<usize> {
        let mut i = 0;
        for (media, _) in self.requests.iter() {
            if media.media_id == media_id {
                return Some(i);
            }
            i += 1;
        }
        None
    }
}
