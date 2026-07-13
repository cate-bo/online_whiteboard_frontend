use serde_json::Value;
use signalr_client::{self, SignalRClient};
use std::option::Option;
use std::{borrow::Cow, hash::Hash, sync::Arc};

use crate::app::OpenWhiteboardResponse;
use crate::http_client_helper::LoginInfo;

pub async fn connect(login_info: Option<LoginInfo>) -> Result<SignalRClient, String> {
    println!("attempting connection");
    SignalRClient::connect_with("localhost", "socket", |cc| {
        cc.with_port(7081);
        cc.secure();
        //cc.with_messagepack_protocol();
        if let Some(info) = login_info.clone() {
            cc.authenticate_bearer(info.accessToken);
        }
    })
    .await
}

pub async fn open_whiteboard(
    mut client: SignalRClient,
    id: i32,
) -> Result<OpenWhiteboardResponse, String> {
    client
        .invoke_with_args("OpenWhiteboard".to_owned(), |c| {
            c.argument(id);
        })
        .await
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
