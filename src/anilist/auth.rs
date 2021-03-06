use anyhow::{anyhow, Result};
use log::debug;
use oauth2::{prelude::SecretNewType, CsrfToken};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};
use url::Url;

pub async fn auth() -> Result<String> {
    let code;
    let state = CsrfToken::new_random().secret().to_string();
    let url_state;
    let client_id = "2355";
    let redirect_uri = "http://localhost:8080/callback";
    let url = Url::parse_with_params(
        "https://anilist.co/api/v2/oauth/authorize",
        &[
            ("client_id", client_id.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("response_type", "code".to_string()),
            ("state", state.to_string()),
        ],
    )?;

    let mut json = HashMap::new();
    json.insert("grant_type", "authorization_code");
    json.insert("client_id", client_id);
    json.insert("redirect_uri", redirect_uri);

    debug!("attempting to open browser to oauth URL");
    open::that(url.to_string())?;

    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        debug!("found ok stream");
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        if let Some(url_code) = request_line.split_whitespace().nth(1) {
            let find_key = |key: &str, url: &Url| {
                if let Some(pair) = url.query_pairs().find(|pair| {
                    let &(ref k, _) = pair;
                    k == key
                }) {
                    pair.1.into_owned()
                } else {
                    String::default()
                }
            };
            let url = format!("http://localhost{}", url_code);
            let url = Url::parse(&url)?;
            code = find_key("code", &url);
            url_state = find_key("state", &url);
        } else {
            code = String::default();
            url_state = String::default();
        }

        if state != url_state {
            return Err(anyhow!(
                "state in oauth redirect was not the same as the generated state"
            ));
        }

        json.insert("code", &code);
        let client = reqwest::Client::new();
        let res = client
            .post("https://auth.fubuki.dev/oauth/token")
            .header("Accept", "application/json")
            .json(&json)
            .send()
            .await?
            .text()
            .await?;

        let msg = "You can close this window now.";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length:{}\r\n\r\n{}",
            msg.len(),
            msg
        );
        stream.write_all(response.as_bytes())?;

        let body: serde_json::Map<String, serde_json::Value> = serde_json::from_str(res.as_str())?;
        if let Some(tok) = body.get("access_token") {
            if let Some(tok) = tok.as_str() {
                return Ok(tok.to_string());
            }
        }
        break;
    }

    Ok(String::default())
}
