use signalr_client::{self, SignalRClient};
use std::option::Option;
use std::{borrow::Cow, hash::Hash, sync::Arc};

use crate::app::Board;
use crate::http_client_helper::LoginInfo;

pub async fn connect(login_info: Option<LoginInfo>) -> Result<SignalRClient, String> {
    println!("attempting connection");
    SignalRClient::connect_with("localhost", "socket", |cc| {
        cc.with_port(7081);
        cc.secure();
        if let Some(info) = login_info.clone() {
            cc.authenticate_bearer(info.accessToken);
        }
    })
    .await
}

pub async fn open_whiteboard(mut client: SignalRClient, id: i32) -> Result<Board, String> {
    client
        .invoke_with_args::<Board, _>("OpenWhiteboard".to_owned(), |c| {
            c.argument(id);
        })
        .await
}

pub async fn test(mut client: SignalRClient, id: i32) -> Result<String, String> {
    client
        .invoke_with_args("OpenWhiteboard".to_owned(), |c| {
            c.argument(id);
        })
        .await
}
