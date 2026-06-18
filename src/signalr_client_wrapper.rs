use signalr_client::{self, SignalRClient};
use std::option::Option;
use std::{borrow::Cow, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

use crate::http_client_wrapper::LoginState::{self, LoggedIn, LoggedOut};

//#[derive(Clone)]
pub struct SignalRClientWrapper {
    pub client: Option<SignalRClient>,
}

impl SignalRClientWrapper {
    pub fn new(login_state: LoginState) -> Self {
        Self { client: None }
    }

    pub async fn connect(&mut self, login_state: LoginState) -> Option<SignalRClient> {
        println!("attempting connection");
        let connection_result = SignalRClient::connect_with("localhost", "socket", |c| {
            if let LoggedIn(info) = login_state.clone() {
                c.authenticate_bearer(info.accessToken);
            }
            c.unsecure();
            c.with_port(5244);
        })
        .await;
        match connection_result {
            Ok(signalr_client) => {
                println!("connected successfully");
                Some(signalr_client)
            }
            Err(error_message) => {
                println!("{}", error_message);
                None
            }
        }
    }
}
