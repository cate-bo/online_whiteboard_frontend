use reqwest::{Response, StatusCode, Url};
use serde_json::{Result, Value};

pub struct HttpClientWrapper {
    client: reqwest::Client,
    pub login_state: LoginState,
    base_url: Url,
    login_url: Url,
    register_url: Url,
    logout_url: Url,
}

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
                let login_info = HttpClientWrapper::parse_token_response(response).await;
                match login_info {
                    Some(info) => {
                        self.login_state = LoginState::LoggedIn(info);
                        println!("successfully logged in");
                    }
                    None => {
                        self.login_state = LoginState::LoginFailed;
                        println!("login failed")
                    }
                }
            }
            Err(_) => {
                self.login_state = LoginState::LoginFailed;
                println!("login failed");
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

    pub async fn attempt_register(&mut self, username: &String, email: &String, password: &String) {
        self.login_state = LoginState::AttemptingRegister;
        println!("attempting register");
        let res = self
            .client
            .post(self.register_url.clone())
            .json(&serde_json::json!({
                "email": email,
                "password": password,
                "username": username,
            }))
            .send()
            .await;
        match res {
            Ok(response) => {
                let login_info = HttpClientWrapper::parse_token_response(response).await;
                match login_info {
                    Some(info) => {
                        self.login_state = LoginState::LoggedIn(info);
                        println!("successfully registered");
                    }
                    None => {
                        self.login_state = LoginState::LoginFailed;
                        println!("register failed");
                    }
                }
            }
            Err(_) => {
                self.login_state = LoginState::RegisterFailed;
                println!("register failed");
            }
        }
    }

    pub async fn logout(&mut self) {
        println!("logging out");
        let res = self
            .client
            .post(self.logout_url.clone())
            .json(&serde_json::json!({ "blabla": "blabla", }))
            .send()
            .await;
        self.login_state = LoginState::LoggedOut;
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
            register_url: temp.join("/register").unwrap(),
            logout_url: temp.join("/logout").unwrap(),
        }
    }
}
#[derive(Clone)]
pub enum LoginState {
    LoggedIn(LoginInfo),
    AttemptingLogin,
    LoginFailed,
    AttemptingRegister,
    RegisterFailed,
    LoggedOut,
}

#[derive(Clone, Default, Debug)]
pub struct LoginInfo {
    pub userName: String,
    pub accessToken: String,
    expiresIn: i64,
    refreshToken: String,
    tokenType: String,
}
