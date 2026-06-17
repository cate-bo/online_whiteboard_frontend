use signalr_client::{self, SignalRClient};

//#[derive(Clone)]
pub struct signalr_client_wrapper {
    client: SignalRClient,
}

impl signalr_client_wrapper {
    pub async fn new() -> Self {
        Self {
            client: SignalRClient::connect_with("localhost", "socket", |o| {})
                .await
                .unwrap(),
        }
    }
}
