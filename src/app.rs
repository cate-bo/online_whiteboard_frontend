use std::{borrow::Cow, hash::Hash, sync::Arc};

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
    show_register_menu: bool,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn login_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let LoginState::AttemptingLogin = self.last_login_state {
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
            if let LoginState::LoginFailed = self.last_login_state {
                ui.label("something went wrong");
            }
            if ui.link("register").clicked() {
                self.show_register_menu = true;
            }
            if ui.button("LOG IN").clicked() {
                let client = self.api_client.clone();
                let username = self.username_inputstring.clone();
                let email = self.email_inputstring.clone();
                let password = self.password_inputstring.clone();
                self.last_login_state = LoginState::AttemptingLogin;
                tokio::task::spawn(async move {
                    client.lock().await.attemt_login(&email, &password).await;
                });
            }
        });
    }

    fn login_or_register_menu(&mut self, ui: &mut egui::Ui) {
        if let LoginState::LoggedIn(_) = self.last_login_state {
            self.email_inputstring = "".to_owned();
            self.password_inputstring = "".to_owned();
            self.username_inputstring = "".to_owned();
            Popup::close_all(ui);
        }
        if (self.show_register_menu) {
            self.register_menu(ui);
        } else {
            self.login_menu(ui);
        }
    }

    fn user_menu(&mut self, ui: &mut egui::Ui) {
        ui.label("worky");
        if (ui.button("LOG OUT").clicked()) {
            let client = self.api_client.clone();
            tokio::task::spawn(async move {
                client.lock().await.logout().await;
            });
        }
    }

    fn register_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let LoginState::AttemptingRegister = self.last_login_state {
            enabled = false;
        }
        ui.add_enabled_ui(enabled, |ui| {
            ui.label("Register:");
            ui.separator();
            ui.label("Username:");
            let username_field = egui::TextEdit::singleline(&mut self.username_inputstring);
            ui.add(username_field);
            ui.label("Email:");
            let email_field = egui::TextEdit::singleline(&mut self.email_inputstring);
            ui.add(email_field);
            ui.label("Password:");
            let password_field =
                egui::TextEdit::singleline(&mut self.password_inputstring).password(true);
            ui.add(password_field);
            if ui.link("log in").clicked() {
                self.show_register_menu = false;
            }
            if ui.button("REGISTER").clicked() {
                let client = self.api_client.clone();
                let username = self.username_inputstring.clone();
                let email = self.email_inputstring.clone();
                let password = self.password_inputstring.clone();
                self.last_login_state = LoginState::AttemptingRegister;
                tokio::task::spawn(async move {
                    client
                        .lock()
                        .await
                        .attempt_register(&username, &email, &password)
                        .await;
                });
            }
        });
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
            show_register_menu: false,
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
                match &self.last_login_state {
                    LoginState::LoggedIn(login_info) => {
                        let user_button_thing = ui.button(&login_info.userName);

                        let mut user_menu = egui::Popup::menu(&user_button_thing);
                        user_menu = user_menu.id(Id::new("user_menu"));
                        user_menu
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .show(|ui| self.user_menu(ui));
                    }
                    _ => {
                        let user_button_thing = ui.button("log in");
                        let mut login_or_register_menu = egui::Popup::menu(&user_button_thing);
                        login_or_register_menu = login_or_register_menu.id(Id::new("login_menu"));
                        login_or_register_menu
                            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                            .show(|ui| self.login_or_register_menu(ui));
                    }
                }
            });
        });
    }
}
