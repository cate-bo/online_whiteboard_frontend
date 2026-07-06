use std::{borrow::Cow, hash::Hash, sync::Arc};

use egui::Plugin;
use egui::{self, Id, Modal, Popup, widgets};
use egui_async::{Bind, EguiAsyncPlugin, StateWithData};
use reqwest::{Client, Error};
use signalrs_client::SignalRClient;

use crate::http_client_helper;
use crate::http_client_helper::{IdAndNameWrapper, LoginInfo};
use crate::signalr_client_helper::{self, connect};

pub struct WhiteboardApp {
    email_inputstring: String,
    username_inputstring: String,
    password_inputstring: String,
    login: Bind<LoginInfo, String>,
    show_register_menu: bool,
    board_name_inputstring: String,
    new_board_is_public: bool,
    new_board_modal_open: bool,
    http_client: Client,
    signalr_client: Bind<SignalRClient, Error>,
    new_board_list: Bind<Vec<IdAndNameWrapper>, String>,
    selected_board: IdAndNameWrapper,
    new_board: Bind<IdAndNameWrapper, String>,
    board_list: Vec<IdAndNameWrapper>,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        Self {
            email_inputstring: "test4@test4.test4".to_owned(),
            username_inputstring: "".to_owned(),
            password_inputstring: "Test4_".to_owned(),
            login: Bind::new(true),
            show_register_menu: false,
            board_name_inputstring: "".to_owned(),
            new_board_is_public: false,
            new_board_modal_open: false,
            http_client: reqwest::Client::new(),
            signalr_client: Bind::new(true),
            new_board_list: Bind::new(true),
            selected_board: IdAndNameWrapper {
                id: 0,
                name: "".to_owned(),
            },
            new_board: Bind::new(true),
            board_list: Vec::new(),
        }
    }

    fn login_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let StateWithData::Pending = self.login.state() {
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
            if let StateWithData::Failed(_) = self.login.state() {
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

                let client = self.http_client.clone();
                self.login.request(async move {
                    http_client_helper::attempt_login(&client, &email, &password).await
                });
            }
        });
    }

    fn login_or_register_menu(&mut self, ui: &mut egui::Ui) {
        if let StateWithData::Finished(_) = self.login.state() {
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
            let client = self.http_client.clone();
            let mut throwaway = Bind::new(false);
            self.login.clear();
            throwaway.request(async move { http_client_helper::logout(&client).await });
        }
    }

    fn register_menu(&mut self, ui: &mut egui::Ui) {
        let mut enabled = true;
        if let StateWithData::Pending = self.login.state() {
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
                let client = self.http_client.clone();
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
                self.login.request(async move {
                    http_client_helper::attempt_register(&client, &username, &email, &password)
                        .await
                });
            }
        });
    }
}

impl eframe::App for WhiteboardApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // egui::MenuBar::new().ui(ui, |ui| {
            ui.horizontal(|ui| {
                //add dropdown for boards
                egui::ComboBox::new("select board", "")
                    .selected_text(self.selected_board.name.clone())
                    .show_ui(ui, |ui| {
                        if let StateWithData::Finished(boards) = self.new_board_list.state() {
                            for board in boards {
                                ui.selectable_value(
                                    &mut self.selected_board,
                                    board.clone(),
                                    &board.name,
                                );
                            }
                            if let StateWithData::Finished(_) = self.login.state() {
                                if ui.button("+").clicked() {
                                    self.new_board_modal_open = true;
                                }
                            }
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("loading boards");
                                ui.add(egui::Spinner::new());
                            });
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
                match &self.login.state() {
                    StateWithData::Finished(info) => {
                        let user_button_thing = ui.button(&info.userName);

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

        if let StateWithData::Finished(info) = self.login.state() {
            if self.new_board_modal_open {
                let modal = Modal::new(Id::new("new_board_modal")).show(ui.ctx(), |ui| {
                    ui.heading("new whiteboard");
                    let mut enabled = true;
                    if let StateWithData::Pending = self.new_board.state() {
                        enabled = false;
                    } else if let StateWithData::Finished(board) = self.new_board.state() {
                        self.board_name_inputstring = "".to_owned();
                        self.new_board_is_public = false;
                        self.selected_board = board.clone();
                        self.board_list.push(board.clone());
                        self.new_board.clear();
                        self.new_board_modal_open = false;
                    }
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.label("name:");
                        ui.text_edit_singleline(&mut self.board_name_inputstring);
                        ui.checkbox(&mut self.new_board_is_public, "public");
                        if ui.button("create").clicked() {
                            let client = self.http_client.clone();
                            let name = self.board_name_inputstring.clone();
                            let token = info.accessToken.clone();
                            let public = self.new_board_is_public.clone();
                            self.new_board.request(async move {
                                http_client_helper::create_board(token, &client, name, public).await
                            });
                        }
                    });
                });
                if modal.should_close() {
                    self.new_board_modal_open = false;
                }
            }
        }
    }
}
