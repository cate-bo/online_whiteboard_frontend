use std::{borrow::Cow, hash::Hash, sync::Arc};

use egui::{self, Id, Modal, Popup, widgets};
use signalr_client::SignalRClient;
use tokio::sync::{Mutex, MutexGuard};

use crate::http_client_wrapper::{self, CreateBoardState, IdAndNameWrapper};
use crate::http_client_wrapper::{HttpClientWrapper, LoginState};
use crate::signalr_client_wrapper::SignalRClientWrapper;
use crate::state_machine::{self, StateMachine};

pub struct WhiteboardApp {
    email_inputstring: String,
    username_inputstring: String,
    password_inputstring: String,
    board_name_inputstring: String,
    new_board_is_public: bool,
    attemting_login: bool,
    state_machine: StateMachine,
    last_login_state: LoginState,
    show_register_menu: bool,
    selected_board: IdAndNameWrapper,
    new_board_modal_open: bool,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        Self {
            email_inputstring: "test4@test4.test4".to_owned(),
            username_inputstring: "".to_owned(),
            password_inputstring: "Test4_".to_owned(),
            board_name_inputstring: "".to_owned(),
            new_board_is_public: false,
            attemting_login: false,
            state_machine: StateMachine::new(),
            last_login_state: LoginState::LoggedOut,
            show_register_menu: false,
            selected_board: IdAndNameWrapper {
                id: 0,
                name: "select board".to_owned(),
            },
            new_board_modal_open: false,
        }
    }

    fn login_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let LoginState::AttemptingLogin = self.state_machine.last_login_state {
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
            if let LoginState::LoginFailed = self.state_machine.last_login_state {
                ui.label("something went wrong");
            }
            if ui.link("register").clicked() {
                self.show_register_menu = true;
            }
            if ui.button("LOG IN").clicked() {
                //let handler = self.state_machine.http_client.clone();
                // let username = self.username_inputstring.clone();
                let email = self.email_inputstring.clone();
                let password = self.password_inputstring.clone();
                // self.state_machine.last_login_state = LoginState::AttemptingLogin;
                // tokio::task::spawn(async move {
                //     handler.lock().await.attempt_login(&email, &password);
                // });
                self.state_machine.attempt_login(email, password);
            }
        });
    }

    fn login_or_register_menu(&mut self, ui: &mut egui::Ui) {
        if let LoginState::LoggedIn(_) = self.state_machine.last_login_state {
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
            let client = self.state_machine.http_client.clone();
            tokio::task::spawn(async move {
                client.lock().await.logout().await;
            });
        }
    }

    fn register_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let LoginState::AttemptingRegister = self.state_machine.last_login_state {
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
                // let client = self.state_machine.http_client.clone();
                let username = self.username_inputstring.clone();
                let email = self.email_inputstring.clone();
                let password = self.password_inputstring.clone();
                // self.state_machine.last_login_state = LoginState::AttemptingRegister;
                // tokio::task::spawn(async move {
                //     client
                //         .lock()
                //         .await
                //         .attempt_register(&username, &email, &password)
                //         .await;
                // });
                self.state_machine
                    .attempt_register(username, email, password);
            }
        });
    }
}

impl eframe::App for WhiteboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.state_machine.update_state();
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // egui::MenuBar::new().ui(ui, |ui| {
            ui.horizontal(|ui| {
                //add dropdown for boards
                egui::ComboBox::new("select board", "")
                    .selected_text(self.selected_board.name.clone())
                    .show_ui(ui, |ui| {
                        for board in self.state_machine.board_list.clone() {
                            ui.selectable_value(
                                &mut self.selected_board,
                                board.clone(),
                                &board.name,
                            );
                        }
                        if let LoginState::LoggedIn(_) = self.state_machine.last_login_state {
                            if ui.button("+").clicked() {
                                self.new_board_modal_open = true;
                            }
                        }
                    });
                ui.menu_button("settings", |ui| {
                    //stuff
                    ui.label("lalala");
                });
                // match self.state_machine.http_client.try_lock() {
                //     Ok(guard) => {
                //         self.state_machine.last_login_state = guard.login_state.clone();
                //     }
                //     _ => {}
                // }
                match &self.state_machine.last_login_state {
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

        if self.new_board_modal_open {
            let modal = Modal::new(Id::new("new_board_modal")).show(ui.ctx(), |ui| {
                ui.heading("new whiteboard");
                let mut enabled = true;
                if let CreateBoardState::Attempting = self.state_machine.create_board_state {
                    enabled = false;
                } else if let CreateBoardState::Success = self.state_machine.create_board_state {
                    self.board_name_inputstring = "".to_owned();
                    self.new_board_is_public = false;
                    self.state_machine.create_board_state = CreateBoardState::None;
                    self.new_board_modal_open = false;
                }
                ui.add_enabled_ui(enabled, |ui| {
                    ui.label("name:");
                    ui.text_edit_singleline(&mut self.board_name_inputstring);
                    ui.checkbox(&mut self.new_board_is_public, "public");
                    if ui.button("create").clicked() {
                        self.state_machine.create_new_board(
                            self.board_name_inputstring.clone(),
                            self.new_board_is_public,
                        );
                    }
                });
            });
            if modal.should_close() {
                self.new_board_modal_open = false;
            }
        }
    }
}
