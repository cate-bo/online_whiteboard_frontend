use reqwest::Url;

use crate::LoginState::{AttemptingLogin, LoggedIn, LoggedOut};

pub struct HttpClientWrapper {
    client: reqwest::Client,
    pub loginState: LoginState,
    base_url: Url,
}

pub enum LoginState {
    LoggedIn,
    AttemptingLogin,
    LoggedOut,
}

impl Clone for LoginState {
    fn clone(&self) -> Self {
        match self {
            LoggedIn => LoggedIn,
            AttemptingLogin => AttemptingLogin,
            LoggedOut => LoggedOut,
        }
    }
}

impl HttpClientWrapper {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn attemt_login(&mut self, email: &String, password: &String) {
        self.loginState = LoginState::AttemptingLogin;
        let res = self
            .client
            .post(self.base_url.clone())
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await;
        match res {
            Ok(response) => {
                self.loginState = LoginState::LoggedIn;
                println!("{response:#?}");
            }
            Err(_) => {
                self.loginState = LoginState::LoggedOut;
                println!("something went wrong")
            }
        }
    }
}

impl Default for HttpClientWrapper {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            loginState: LoginState::LoggedOut,
            base_url: reqwest::Url::parse("https://localhost:7081/").unwrap(),
        }
    }
}
