use super::models::{MediaListCollection, MediaType, User, MediaList};
use anyhow::{anyhow, Result};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Map, Value};
use std::path::Path;
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

pub async fn query_from_file<R, P>(
    path: P,
    variables: &Option<Map<String, Value>>,
    token: Option<String>,
) -> Result<QueryResponse<R>>
where
    R: DeserializeOwned,
    P: AsRef<Path>,
{
    let query = std::fs::read_to_string(path)?;
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
        query_from_file("./res/graphql/media_list.gql", &Some(variables), token).await
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
    query_from_file("./res/graphql/user.gql", &None, token).await
}

pub async fn update_media(token: Option<String>, media: MediaList) -> Result<QueryResponse<SaveMediaListEntryResponse>> {
    let variables = json!({
        "id": media.id,
        "status": media.status,
        "progress": media.progress,
        "progressVolumes": media.progress_volumes,
        "startedAt": media.started_at,
        "completedAt": media.completed_at,
    });
    if let serde_json::Value::Object(variables) = variables {
        query_from_file("./res/graphql/update_media.gql", &Some(variables), token).await
    } else {
        Err(anyhow!("update media variables was not a json object"))
    }
}