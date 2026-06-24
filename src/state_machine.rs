use clone;
use reqwest::{Response, StatusCode, Url};
use serde_json::{Result, Value};
use signalr_client::SignalRClient;
use std::{borrow::Cow, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    http_client_wrapper::{
        HttpClientWrapper,
        LoginState::{self, AttemptingLogin, AttemptingRegister, LoggedIn, LoggedOut},
    },
    signalr_client_wrapper::SignalRClientWrapper,
};

pub struct StateMachine {
    pub http_client: Arc<Mutex<HttpClientWrapper>>,
    pub last_login_state: LoginState,
    pub signalr_client: Arc<Mutex<SignalRClientWrapper>>,
    pub connected: bool,
}

impl StateMachine {
    pub fn new() -> Self {
        let temp = Self {
            http_client: Arc::new(Mutex::new(HttpClientWrapper::new())),
            last_login_state: LoggedOut,
            signalr_client: Arc::new(Mutex::new(SignalRClientWrapper::new(LoggedOut))),
            connected: false,
        };
        temp.connect();
        // let pointer = temp.signalr_client.clone();
        // tokio::task::spawn(async move {
        //     pointer.lock().await.connect(LoggedOut).await;
        // });
        return temp;
    }

    pub fn update_state(&mut self) {
        if let Ok(guard) = self.http_client.try_lock() {
            let mut currently_logged_in = false;
            if let LoggedIn(_) = guard.login_state.clone() {
                currently_logged_in = true;
            }
            let mut previously_logged_in = false;
            if let LoggedIn(_) = self.last_login_state {
                previously_logged_in = true;
            }
            match (currently_logged_in, previously_logged_in) {
                (true, false) => {}
                (false, true) => {}
                _ => {}
            }
            self.last_login_state = guard.login_state.clone();
        }
        if let Ok(guard) = self.signalr_client.try_lock() {
            if let Some(_) = guard.client {
                self.connected = true;
            } else {
                self.connected = false;
            }
        }
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
}
