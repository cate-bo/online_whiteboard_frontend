use std::{hash::Hash, sync::Arc};

use egui::{self, Id, Popup, widgets};
use tokio::sync::{Mutex, MutexGuard};

use crate::api_helper::{HttpClientWrapper, LoginState};

pub struct WhiteboardApp {
    email_inputstring: String,
    username_inputstring: String,
    password_inputstring: String,
    attemting_login: bool,
    api_client: Arc<Mutex<HttpClientWrapper>>,
    last_login_state: LoginState,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn login_menu(&mut self, ui: &mut egui::Ui) {
        if let LoginState::LoggedIn(_) = self.last_login_state {
            Popup::close_all(ui);
            return;
        }
        let mut enabled = true;
        if let LoginState::AttemptingLogin = self.last_login_state.clone() {
            enabled = false;
        }
        ui.add_enabled_ui(enabled, |ui| {
            ui.label("Log in:");
            ui.separator();
            ui.label("Email:");
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
                self.last_login_state = LoginState::AttemptingLogin;
                tokio::task::spawn(async move {
                    client.lock().await.attemt_login(&email, &password).await;
                });
            }
        });
    }

    fn user_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("worky");
    }
}

impl Default for WhiteboardApp {
    fn default() -> Self {
        Self {
            email_inputstring: "test@test.test".to_owned(),
            username_inputstring: "".to_owned(),
            password_inputstring: "Test1_".to_owned(),
            attemting_login: false,
            api_client: Arc::new(Mutex::new(HttpClientWrapper::new())),
            last_login_state: LoginState::LoggedOut,
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
                        self.last_login_state = guard.login_state.clone();
                    }
                    _ => {}
                }
                let user_button_thing = ui.button("log in");
                match self.last_login_state {
                    LoginState::LoggedIn(_) => {
                        let mut user_menu = egui::Popup::menu(&user_button_thing);
                        user_menu = user_menu.id(Id::new("user_menu"));
                        user_menu
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .show(|ui| self.user_menu(ui));
                    }
                    _ => {
                        let mut login_menu = egui::Popup::menu(&user_button_thing);
                        login_menu = login_menu.id(Id::new("login_menu"));
                        login_menu
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .show(|ui| self.login_menu(ui));
                    }
                }
            });
        });
    }
}
