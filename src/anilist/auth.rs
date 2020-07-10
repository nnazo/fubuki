use url::Url;
use std::{sync::Mutex, net::{TcpListener}, io::{BufRead, BufReader, Write}, collections::HashMap};
use oauth2::{CsrfToken, prelude::SecretNewType};
use anyhow::Result;

pub async fn auth() -> Result<String> {
    let code;
    let state = CsrfToken::new_random().secret().to_string();
    let urlState;
    let client_id = "2355";
    let redirect_uri = "http://localhost:8080/callback";
    let url = Url::parse_with_params(
        "https://anilist.co/api/v2/oauth/authorize", 
        &[
            ("client_id", client_id.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("response_type", "code".to_string()),
            ("state", state.to_string()),
        ]
    )?;

    let mut json = HashMap::new();
    json.insert("grant_type", "authorization_code");
    json.insert("client_id", client_id);
    json.insert("redirect_uri", redirect_uri);

    println!("attempting to open browser");
    if open::that(url.to_string()).is_err() {
        println!("browser failed to open");
    };

    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("found ok stream");
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line)?;

                // println!("{}", request_line);

                if let Some(url_code) = request_line.split_whitespace().nth(1) {
                    let url = Url::parse(&("http://localhost".to_string() + url_code))?;
                    if let Some(code_pair) = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "code"
                        }) 
                    {
                        code = code_pair.1.into_owned();
                    } else {
                        code = "".to_string();
                    };

                    if let Some(state_pair) = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "state"
                        })
                    {
                        urlState = state_pair.1.into_owned();
                    } else {
                        urlState = "".to_string();
                    };
                } else {
                    code = "".to_string();
                    urlState = "".to_string();
                }

                // if state == urlState {

                // }

                json.insert("code", code.as_str());
                let client = reqwest::Client::new();
                let res = client.post("http://localhost:8081/oauth/token")
                    .header("Accept", "application/json")
                    .json(&json)
                    .send()
                    .await?
                    .text()
                    .await?;
                // println!("res: {}", res);

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
            Err(e) => {
                println!("{}", e);
                break;
            },
        }
    }

    Ok("".to_string())
    
}
