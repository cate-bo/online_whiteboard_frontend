use reqwest::{Response, StatusCode, Url};
use serde_json::{
    Result,
    Value::{self, Array},
    value,
};

pub struct HttpClientWrapper {
    client: reqwest::Client,
    pub login_state: LoginState,
    pub create_board_state: CreateBoardState,
    pub boards: Vec<IdAndNameWrapper>,
    pub boards_changed: bool,
    base_url: Url,
    login_url: Url,
    register_url: Url,
    logout_url: Url,
    board_url: Url,
}

impl HttpClientWrapper {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn give_boards(&mut self) -> Vec<IdAndNameWrapper> {
        self.boards_changed = false;
        self.boards.clone()
    }

    pub async fn attempt_login(&mut self, email: &String, password: &String) -> LoginState {
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
                        println!("login failed");
                    }
                }
            }
            Err(_) => {
                self.login_state = LoginState::LoginFailed;
                println!("login failed");
            }
        }
        self.login_state.clone()
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
        &mut self,
        username: &String,
        email: &String,
        password: &String,
    ) -> LoginState {
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

        self.login_state.clone()
    }

    pub async fn logout(&mut self) {
        println!("logging out");
        let res = self
            .client
            .post(self.logout_url.clone())
            .json(&serde_json::json!({ "blabla": "blabla", }))
            .send()
            .await;
        match res {
            Ok(_) => {
                println!("all good");
            }
            Err(_) => {
                println!("something brokie");
            }
        }
        self.login_state = LoginState::LoggedOut;
    }

    pub async fn create_board(&mut self, name: String, public: bool) {
        if let LoginState::LoggedIn(info) = self.login_state.clone() {
            println!("attempting board creation");
            self.create_board_state = CreateBoardState::Attempting;
            let res = self
                .client
                .post(self.board_url.clone())
                .json(&serde_json::json!({
                    "name": name,
                    "isPublic": public,
                }))
                .header("Authorization", "Bearer ".to_owned() + &info.accessToken)
                .send()
                .await;
            match res {
                Ok(response) => {
                    if (response.status() != StatusCode::OK) {
                        self.create_board_state = CreateBoardState::Failiure;
                        return;
                    } else {
                        self.create_board_state = CreateBoardState::Success;
                        return;
                    }
                }
                Err(_) => {
                    self.create_board_state = CreateBoardState::Failiure;
                }
            }
        } else {
            self.create_board_state = CreateBoardState::Failiure;
        }
    }

    pub async fn get_board_list(&mut self) {
        println!("fetching board list");
        let mut builder = self.client.get(self.board_url.clone());
        if let LoginState::LoggedIn(info) = self.login_state.clone() {
            builder = builder.header("Authorization", "Bearer ".to_owned() + &info.accessToken);
        }
        let res = builder.send().await;
        match res {
            Ok(response) => {
                self.boards.clear();
                let response_text = response.text().await.unwrap();
                let response_body: Value = serde_json::from_str(&response_text).unwrap();
                if let Array(values) = response_body {
                    for value in values {
                        if let Value::Object(map) = value {
                            let mut board: IdAndNameWrapper = IdAndNameWrapper {
                                id: 0,
                                name: "".to_owned(),
                            };
                            board.id =
                                i32::try_from(map.get("id").unwrap().as_i64().unwrap()).unwrap();
                            board.name = map.get("name").unwrap().to_string();
                            self.boards.push(board);
                        }
                    }
                }
                self.boards_changed = true;
            }
            Err(_) => {}
        }
    }
}

impl Default for HttpClientWrapper {
    fn default() -> Self {
        let temp = reqwest::Url::parse("https://localhost:7081/").unwrap();
        Self {
            client: reqwest::Client::new(),
            login_state: LoginState::LoggedOut,
            create_board_state: CreateBoardState::None,
            boards: Vec::new(),
            boards_changed: false,
            base_url: temp.clone(),
            login_url: temp.join("/login").unwrap(),
            register_url: temp.join("/register").unwrap(),
            logout_url: temp.join("/logout").unwrap(),
            board_url: temp.join("/boards").unwrap(),
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

#[derive(Clone)]
pub enum CreateBoardState {
    Success,
    Attempting,
    Failiure,
    None,
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
