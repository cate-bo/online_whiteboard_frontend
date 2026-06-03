use std::sync::Arc;

use egui::{self, widgets};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    LoginState::AttemptingLogin,
    api_helper::{HttpClientWrapper, LoginState},
};

pub struct WhiteboardApp {
    email_inputstring: String,
    username_inputstring: String,
    password_inputstring: String,
    attemting_login: bool,
    api_client: Arc<Mutex<HttpClientWrapper>>,
    last_loginState: LoginState,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn login_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let AttemptingLogin = self.last_loginState {
            enabled = false;
        }
        ui.add_enabled_ui(enabled, |ui| {
            ui.label("Log in:");
            ui.separator();
            ui.label("User Name:");
            let email_field = egui::TextEdit::singleline(&mut self.email_inputstring);
            ui.add(email_field);
            ui.label("Password:");
            let password_field =
                egui::TextEdit::singleline(&mut self.password_inputstring).password(true);
            ui.add(password_field);
            if ui.button("LOG IN").clicked() {
                let client = self.api_client.clone();
                let email = self.email_inputstring.clone();
                let password = self.password_inputstring.clone();
                println!("amogus");
                tokio::task::spawn(async move {
                    client.lock().await.attemt_login(&email, &password).await;
                });
            }
        });
    }

    fn user_menu(&mut self, ui: &mut egui::Ui) {}
}

impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            email_inputstring: "".to_owned(),
            username_inputstring: "".to_owned(),
            password_inputstring: "".to_owned(),
            attemting_login: false,
            api_client: Arc::new(Mutex::new(HttpClientWrapper::new())),
            last_loginState: LoginState::LoggedOut,
        }
    }
}

impl eframe::App for WhiteboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("settings", |ui| {
                    //stuff
                    ui.label("lalala");
                });
                match self.api_client.try_lock() {
                    Ok(guard) => {
                        self.last_loginState = guard.loginState.clone();
                    }
                    _ => {}
                }
                match self.last_loginState {
                    LoginState::LoggedIn => {}
                    _ => {
                        egui::Popup::menu(&mut ui.button("log in"))
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .show(|ui| self.login_menu(ui));
                    }
                }
            });
        });
    }
}
