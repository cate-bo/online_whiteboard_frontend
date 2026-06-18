use signalrs_client::builder::Auth;
use signalrs_client::hub::Hub;
use signalrs_client::{self, SignalRClient};
use std::option::Option;
use std::{borrow::Cow, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

use crate::http_client_wrapper::LoginState::{self, LoggedIn, LoggedOut};

//#[derive(Clone)]
pub struct SignalRClientWrapper {
    pub client: Option<SignalRClient>,
}
async fn test(message: String) {
    println!("{}", message);
}

impl SignalRClientWrapper {
    pub fn new(login_state: LoginState) -> Self {
        Self { client: None }
    }

    pub async fn connect(&mut self, login_state: LoginState) -> Result<(), ()> {
        println!("attempting connection");
        let client_hub = Hub::default().method("test", test);
        let connection_result = SignalRClient::builder("localhost")
            .use_hub("socket")
            .use_port(7081)
            .with_client_hub(client_hub)
            .build()
            .await;
        // let connection_result = SignalRClient::connect_with("localhost", "socket", |c| {
        //     if let LoggedIn(info) = login_state.clone() {
        //         c.authenticate_bearer(info.accessToken);
        //     }
        //     c.unsecure();
        //     c.with_port(5244);
        // })
        // .await;
        match connection_result {
            Ok(signalr_client) => {
                println!("connected successfully");
                let text = signalr_client
                    .method("Test")
                    .arg("blabla")
                    .unwrap()
                    .invoke::<String>()
                    .await;
                self.client = Some(signalr_client);
                Ok(())
            }
            Err(error_message) => {
                println!("{}", error_message);
                self.client = None;
                Err(())
            }
        }
    }
}
