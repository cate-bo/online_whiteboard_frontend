use clone;
use reqwest::{Response, StatusCode, Url};
use serde_json::{Result, Value};
use signalr_client::SignalRClient;
use std::{borrow::Cow, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    http_client_wrapper::{
        HttpClientWrapper,
        LoginState::{self, LoggedOut},
    },
    signalr_client_wrapper::SignalRClientWrapper,
};

pub struct NetworkHandler {
    pub http_client: Arc<Mutex<HttpClientWrapper>>,
    pub last_login_state: LoginState,
    pub signalr_client: Arc<Mutex<SignalRClientWrapper>>,
}

impl NetworkHandler {
    pub fn new() -> Self {
        let temp = Self {
            http_client: Arc::new(Mutex::new(HttpClientWrapper::new())),
            last_login_state: LoggedOut,
            signalr_client: Arc::new(Mutex::new(SignalRClientWrapper::new(LoggedOut))),
        };
        let pointer = temp.signalr_client.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.connect(LoggedOut).await;
        });
        return temp;
    }

    pub async fn connect(&self, login_state: LoginState) {
        let pointer = self.signalr_client.clone();
        let pointer2 = pointer.clone();
        tokio::task::spawn(async move {
            pointer.lock().await.connect(LoggedOut).await;
        })
        .await;
        if let Some(_) = pointer2.lock().await.client {}
    }

    pub async fn attempt_login(&mut self, email: String, password: String) {
        let pointer = self.http_client.clone();
        self.last_login_state = pointer.lock().await.attempt_login(&email, &password).await;
    }

    pub async fn update_last_login_state(&mut self) {}
}
