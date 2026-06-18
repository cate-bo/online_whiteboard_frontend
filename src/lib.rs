#![allow(warnings)]

mod app;
pub use app::WhiteboardApp;
mod login_menu;
pub use login_menu::Login_menu;
mod state_machine;
//pub use network_handler::{LoginState, http_client_wrapper};
mod signalr_client_wrapper;
//pub use socket_helper;
mod http_client_wrapper;
