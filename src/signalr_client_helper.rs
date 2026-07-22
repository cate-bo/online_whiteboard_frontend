use serde_json::{Value, json};
use signalr_client::{
    self, DisconnectionHandler, NoReconnectPolicy, ReconnectionConfig, SignalRClient,
};
use std::option::Option;
use std::{borrow::Cow, hash::Hash, sync::Arc, sync::mpsc::Sender};

use crate::app::OpenWhiteboardResponse;
use crate::app::{Update, Whiteboard};
use crate::http_client_helper::LoginInfo;

pub async fn connect(
    login_info: Option<LoginInfo>,
    mpsc_sender: Sender<Update>,
) -> Result<SignalRClient, String> {
    println!("attempting connection");
    let client = SignalRClient::connect_with("localhost", "socket", |cc| {
        cc.with_port(7081);
        cc.secure();
        cc.with_reconnection_policy(ReconnectionConfig {
            policy: Arc::new(NoReconnectPolicy),
        });
        cc.with_disconnection_handler(DCHandler {});
        //cc.with_messagepack_protocol();
        if let Some(info) = login_info.clone() {
            println!("have token");
            cc.authenticate_bearer(info.accessToken);
        }
    })
    .await?;

    //register callbacks here
    return Ok(client);
}

pub async fn open_whiteboard(mut client: SignalRClient, id: i32, sender: Sender<Update>) {
    let test: Result<String, String> = client
        .invoke_with_args("OpenWhiteboard".to_owned(), |c| {
            c.argument(id);
        })
        .await;
    let mut res = Err("".to_owned());
    if let Ok(thing) = test {
        // println!("{}", thing);
        let stuff: OpenWhiteboardResponse = serde_json::from_str(&thing).unwrap();
        res = Ok(stuff);
        println!("{:?}", thing);
    }
    // let res = client
    //     .invoke_with_args("OpenWhiteboard".to_owned(), |c| {
    //         c.argument(id);
    //     })
    //     .await;
    match res {
        Ok(resp) => {
            sender.send(Update::Boardrecieved(resp));
        }
        Err(msg) => {
            sender.send(Update::BoardError);
        }
    }
}

pub async fn test(mut client: SignalRClient) -> Result<Value, String> {
    client
        .invoke_with_args("OpenWhiteboard".to_owned(), |c| {
            c.argument(1);
        })
        .await
}

pub async fn test2(mut client: SignalRClient) -> Result<String, String> {
    client.invoke("Test2".to_owned()).await
}

struct DCHandler {}
impl DisconnectionHandler for DCHandler {
    fn on_disconnected(&self, reconnection: signalr_client::ReconnectionHandler) {}
}
