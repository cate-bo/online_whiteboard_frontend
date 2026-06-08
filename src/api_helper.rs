use reqwest::Url;

pub struct HttpClientWrapper {
    client: reqwest::Client,
    pub login_state: LoginState,
    base_url: Url,
    login_url: Url,
}

pub struct User {}

pub enum LoginState {
    LoggedIn,
    AttemptingLogin,
    LoginFailed,
    LoggedOut,
}

impl Clone for LoginState {
    fn clone(&self) -> Self {
        match self {
            LoginState::LoggedIn => LoginState::LoggedIn,
            LoginState::AttemptingLogin => LoginState::AttemptingLogin,
            LoginState::LoginFailed => LoginState::LoginFailed,
            LoginState::LoggedOut => LoginState::LoggedOut,
        }
    }
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
                self.login_state = LoginState::LoggedIn;
                let thing: String = response.text().await.unwrap();
                println!("{thing:#?}");
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
