use clone;
use reqwest::Url;
use serde_json::{Result, Value};

pub struct HttpClientWrapper {
    client: reqwest::Client,
    pub login_state: LoginState,
    base_url: Url,
    login_url: Url,
}

pub struct User {}

#[derive(Clone)]
pub enum LoginState {
    LoggedIn(LoginInfo),
    AttemptingLogin,
    LoginFailed,
    LoggedOut,
}

/*
impl Clone for LoginState {
    fn clone(&self) -> Self {
        match self {
            LoginState::LoggedIn(info) => LoginState::LoggedIn(info.clone()),
            LoginState::AttemptingLogin => LoginState::AttemptingLogin,
            LoginState::LoginFailed => LoginState::LoginFailed,
            LoginState::LoggedOut => LoginState::LoggedOut,
        }
    }
}
*/

impl HttpClientWrapper {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn attemt_login(&mut self, email: &String, password: &String) {
        self.login_state = LoginState::AttemptingLogin;
        println!("attempting login");
        let res = self
            .client
            .post(self.login_url.clone())
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await;
        match res {
            Ok(response) => {
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
                //let bla = login_info.accessToken;
                self.login_state = LoginState::LoggedIn(login_info);
                println!("successfully logged in");
            }
            Err(_) => {
                self.login_state = LoginState::LoggedOut;
                println!("something went wrong");
            }
        }
    }
}

impl Default for HttpClientWrapper {
    fn default() -> Self {
        let temp = reqwest::Url::parse("https://localhost:7081/").unwrap();
        Self {
            client: reqwest::Client::new(),
            login_state: LoginState::LoggedOut,
            base_url: temp.clone(),
            login_url: temp.join("/login").unwrap(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct LoginInfo {
    userName: String,
    accessToken: String,
    expiresIn: i64,
    refreshToken: String,
    tokenType: String,
}
