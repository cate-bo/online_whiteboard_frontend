use clone;
use reqwest::{Response, StatusCode, Url};
use serde_json::{Result, Value};
use signalr_client::SignalRClient;
use std::{borrow::Cow, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    http_client_wrapper::{HttpClientWrapper, LoginState::LoggedOut},
    signalr_client_wrapper::SignalRClientWrapper,
};

pub struct NetworkHandler {
    pub http_client: Arc<Mutex<HttpClientWrapper>>,
    pub signalr_client: Arc<Mutex<SignalRClientWrapper>>,
}

impl NetworkHandler {
    pub fn new() -> Self {
        Self {
            http_client: Arc::new(Mutex::new(HttpClientWrapper::new())),
            signalr_client: Arc::new(Mutex::new(SignalRClientWrapper::new(LoggedOut))),
        }
    }
}
