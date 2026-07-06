use std::result::Result;

use reqwest::{Client, Response, StatusCode, Url};
use serde_json::{
    Value::{self, Array},
    value,
};

const base_url: &str = "https://localhost:7081/";
const login_url: &str = "https://localhost:7081/login";
const register_url: &str = "https://localhost:7081/register";
const logout_url: &str = "https://localhost:7081/logout";
const board_url: &str = "https://localhost:7081/boards";

pub async fn attempt_login(
    client: &Client,
    email: &String,
    password: &String,
) -> Result<LoginInfo, String> {
    println!("attempting login");
    let res = client
        .post(login_url)
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await;
    match res {
        Ok(response) => {
            let login_info = parse_token_response(response).await;
            match login_info {
                Some(info) => {
                    return Ok(info);
                }
                None => {
                    return Err("invalid credentials".to_owned());
                }
            }
        }
        Err(_) => {
            return Err("connection error".to_owned());
        }
    }
}

pub async fn parse_token_response(response: Response) -> Option<LoginInfo> {
    if (response.status() != StatusCode::OK) {
        return None;
    }
    let mut login_info = LoginInfo::default();
    login_info.userName = response
        .headers()
        .iter()
        .find(|header| header.0.eq("name"))
        .unwrap()
        .1
        .to_str()
        .unwrap()
        .to_owned();
    let thing: String = response.text().await.unwrap().clone();
    let response_body: Value = serde_json::from_str(&thing).unwrap();
    login_info.accessToken = response_body
        .get("accessToken")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_owned();
    login_info.expiresIn = response_body.get("expiresIn").unwrap().as_i64().unwrap();
    login_info.refreshToken = response_body
        .get("refreshToken")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_owned();
    login_info.tokenType = response_body
        .get("tokenType")
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_owned();
    return Some(login_info);
}

pub async fn attempt_register(
    client: &Client,
    username: &String,
    email: &String,
    password: &String,
) -> Result<LoginInfo, String> {
    println!("attempting register");
    let res = client
        .post(register_url)
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "username": username,
        }))
        .send()
        .await;
    match res {
        Ok(response) => {
            let login_info = parse_token_response(response).await;
            match login_info {
                Some(info) => {
                    return Ok(info);
                }
                None => {
                    return Err("invalid userdata".to_owned());
                }
            }
        }
        Err(_) => {
            return Err("connection error".to_owned());
        }
    }
}

pub async fn logout(client: &Client) -> Result<(), ()> {
    println!("logging out");
    let res = client
        .post(logout_url)
        .json(&serde_json::json!({ "blabla": "blabla", }))
        .send()
        .await;
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub async fn create_board(
    accessToken: String,
    client: &Client,
    name: String,
    public: bool,
) -> Result<IdAndNameWrapper, String> {
    let res = client
        .post(board_url)
        .json(&serde_json::json!({
            "name": name,
            "isPublic": public,
        }))
        .header("Authorization", "Bearer ".to_owned() + &accessToken)
        .send()
        .await;
    match res {
        Ok(response) => {
            if (response.status() != StatusCode::OK) {
                let response_text = response.text().await.unwrap();
                let response_body: Value = serde_json::from_str(&response_text).unwrap();
                if let Value::Object(map) = response_body {
                    let mut board: IdAndNameWrapper = IdAndNameWrapper {
                        id: 0,
                        name: "".to_owned(),
                    };
                    board.id = i32::try_from(map.get("id").unwrap().as_i64().unwrap()).unwrap();
                    board.name = map.get("name").unwrap().to_string();
                    return Ok(board);
                } else {
                    return Err("board creation failed".to_owned());
                }
            } else {
                return Err("connection error".to_owned());
            }
        }
        Err(_) => {
            return Err("connection error".to_owned());
        }
    }
}

pub async fn get_board_list(
    client: &Client,
    accessToken: Option<String>,
) -> Result<Vec<IdAndNameWrapper>, String> {
    let mut builder = client.get(board_url);
    if let Some(token) = accessToken {
        builder = builder.header("Authorization", "Bearer ".to_owned() + &token);
    }
    let res = builder.send().await;
    match res {
        Ok(response) => {
            let mut boards = Vec::new();
            let response_text = response.text().await.unwrap();
            let response_body: Value = serde_json::from_str(&response_text).unwrap();
            if let Array(values) = response_body {
                for value in values {
                    if let Value::Object(map) = value {
                        let mut board: IdAndNameWrapper = IdAndNameWrapper {
                            id: 0,
                            name: "".to_owned(),
                        };
                        board.id = i32::try_from(map.get("id").unwrap().as_i64().unwrap()).unwrap();
                        board.name = map.get("name").unwrap().to_string();
                        boards.push(board);
                    }
                }
            }
            return Ok(boards);
        }
        Err(_) => return Err("connection error".to_owned()),
    }
}

#[derive(Clone, Default, Debug)]
pub struct LoginInfo {
    pub userName: String,
    pub accessToken: String,
    expiresIn: i64,
    refreshToken: String,
    tokenType: String,
}

#[derive(Clone)]
pub struct IdAndNameWrapper {
    pub id: i32,
    pub name: String,
}

impl PartialEq for IdAndNameWrapper {
    fn eq(&self, other: &Self) -> bool {
        if (self.id == other.id) { true } else { false }
    }
}
