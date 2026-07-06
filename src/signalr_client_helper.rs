use signalrs_client::builder::{Auth, BuilderError};
use signalrs_client::hub::Hub;
use signalrs_client::{self, SignalRClient};
use std::option::Option;
use std::{borrow::Cow, hash::Hash, sync::Arc};

use crate::http_client_helper::LoginInfo;

pub async fn connect(login_info: Option<LoginInfo>) -> Result<SignalRClient, BuilderError> {
    println!("attempting connection");
    let client_hub = Hub::default();
    let mut builder = SignalRClient::builder("localhost")
        .use_hub("socket")
        .use_port(7081)
        .with_client_hub(client_hub);

    if let Some(info) = login_info {
        let auth = Auth::Bearer {
            token: info.accessToken,
        };
        builder = builder.use_authentication(auth);
    }

    builder.build().await
}
