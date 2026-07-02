use clone;
use reqwest::{Response, StatusCode, Url};
use serde_json::{Result, Value};
use signalr_client::SignalRClient;
use std::{borrow::Cow, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    http_client_wrapper::{
        CreateBoardState, HttpClientWrapper, IdAndNameWrapper,
        LoginState::{self, AttemptingLogin, AttemptingRegister, LoggedIn, LoggedOut},
    },
    signalr_client_wrapper::SignalRClientWrapper,
};

pub struct StateMachine {
    pub http_client: Arc<Mutex<HttpClientWrapper>>,
    pub last_login_state: LoginState,
    pub create_board_state: CreateBoardState,
    pub signalr_client: Arc<Mutex<SignalRClientWrapper>>,
    pub connected: bool,
    pub board_list: Vec<IdAndNameWrapper>,
}

impl StateMachine {
    pub fn new() -> Self {
        let mut temp = Self {
            http_client: Arc::new(Mutex::new(HttpClientWrapper::new())),
            last_login_state: LoggedOut,
            create_board_state: CreateBoardState::None,
            signalr_client: Arc::new(Mutex::new(SignalRClientWrapper::new(LoggedOut))),
            connected: false,
            board_list: Vec::new(),
        };
        temp.connect();
        temp.refresh_board_list();
        // let pointer = temp.signalr_client.clone();
        // tokio::task::spawn(async move {
        //     pointer.lock().await.connect(LoggedOut).await;
        // });
        return temp;
    }

    pub fn update_state(&mut self) {
        let mut currently_logged_in = false;
        let mut previously_logged_in = false;
        if let Ok(mut guard) = self.http_client.try_lock() {
            if let LoggedIn(_) = guard.login_state.clone() {
                currently_logged_in = true;
            }
            if let LoggedIn(_) = self.last_login_state {
                previously_logged_in = true;
            }
            self.last_login_state = guard.login_state.clone();
            if guard.boards_changed {
                self.board_list = guard.give_boards();
            }
        }
        match (currently_logged_in, previously_logged_in) {
            (true, false) | (false, true) => {
                self.refresh_board_list();
            }
            _ => {}
        }
        if let Ok(guard) = self.signalr_client.try_lock() {
            if let Some(_) = guard.client {
                self.connected = true;
            } else {
                self.connected = false;
            }
        }
        // if self.board_list_changed {
        //     if let Ok(guard) = self.http_client.try_lock() {
        //         self.board_list = guard.boards.clone();
        //         self.board_list_changed = false;
        //     }
        // }
        if let CreateBoardState::Attempting = self.create_board_state {
            if let Ok(guard) = self.http_client.try_lock() {
                self.create_board_state = guard.create_board_state.clone();
            }
            if let CreateBoardState::Success = self.create_board_state {
                self.refresh_board_list();
            } else if let CreateBoardState::Failiure = self.create_board_state {
            }
        }
    }

    pub fn refresh_board_list(&mut self) {
        let pointer = self.http_client.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.get_board_list().await;
        });
    }

    pub async fn connect(&self) {
        let pointer = self.signalr_client.clone();
        let login_state = self.last_login_state.clone();
        // let pointer2 = pointer.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.connect(login_state).await;
        });
        // if let Some(_) = pointer2.lock().await.client {}
    }

    pub fn attempt_login(&mut self, email: String, password: String) {
        self.last_login_state = AttemptingLogin;
        let pointer = self.http_client.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.attempt_login(&email, &password).await;
        });
    }

    pub fn attempt_register(&mut self, username: String, email: String, password: String) {
        self.last_login_state = AttemptingRegister;
        let pointer = self.http_client.clone();
        tokio::task::spawn(async move {
            pointer
                .lock()
                .await
                .attempt_register(&username, &email, &password)
                .await;
        });
    }

    pub fn logout(&mut self) {
        self.last_login_state = LoggedOut;
        let pointer = self.http_client.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.logout().await;
        });
    }

    pub fn create_new_board(&mut self, name: String, public: bool) {
        self.create_board_state = CreateBoardState::Attempting;
        let pointer = self.http_client.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.create_board(name, public).await;
        });
    }
}
