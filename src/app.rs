use std::{borrow::Cow, hash::Hash, sync::Arc};

use egui::Plugin;
use egui::{self, Id, Modal, Popup, widgets};
use egui_async::{Bind, EguiAsyncPlugin, StateWithData};
use reqwest::{Client, Error};
use serde::{Deserialize, de};
use signalr_client::SignalRClient;

use crate::http_client_helper;
use crate::http_client_helper::{IdAndNameWrapper, LoginInfo};
use crate::signalr_client_helper::{self};

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
    signalr_client: Bind<SignalRClient, String>,
    new_board_list: Bind<Vec<IdAndNameWrapper>, String>,
    selected_board: IdAndNameWrapper,
    new_board: Bind<IdAndNameWrapper, String>,
    board_list: Vec<IdAndNameWrapper>,
    previously_logged_in: bool,
    opened_board: Bind<String, String>,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        let mut temp = Self {
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
            previously_logged_in: false,
            opened_board: Bind::new(true),
        };
        temp.refresh_boards();
        temp.connect_signalr();
        return temp;
    }

    fn login_changed(&mut self) {
        self.refresh_boards();
        self.connect_signalr();
    }

    fn connect_signalr(&mut self) {
        println!("trying to connect to signalr");
        let mut info: Option<LoginInfo> = None;
        if let StateWithData::Finished(login_info) = self.login.state() {
            info = Some(login_info.clone());
        }
        self.signalr_client
            .request(async move { signalr_client_helper::connect(info).await })
    }

    fn refresh_boards(&mut self) {
        let client = self.http_client.clone();
        let mut accessToken: Option<String> = None;
        if let StateWithData::Finished(info) = self.login.state() {
            accessToken = Some(info.accessToken.clone());
        }
        self.new_board_list
            .request(async move { http_client_helper::get_board_list(&client, accessToken).await });
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
                let email = self.email_inputstring.clone();
                let password = self.password_inputstring.clone();
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
            self.login_changed();
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
            self.login_changed();
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
        let mut currently_logged_in = false;
        if let StateWithData::Finished(_) = self.login.state() {
            currently_logged_in = true;
        }
        if (self.previously_logged_in ^ currently_logged_in) {
            self.login_changed();
            self.previously_logged_in = currently_logged_in;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let previous_board = self.selected_board.clone();
        if let StateWithData::Finished(new_boards) = self.new_board_list.state() {
            self.board_list = new_boards.clone();
            self.new_board_list.clear();
        }
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                //add dropdown for boards
                egui::ComboBox::new("select board", "")
                    .selected_text(self.selected_board.name.clone())
                    .show_ui(ui, |ui| {
                        if let StateWithData::Pending = self.new_board_list.state() {
                            ui.horizontal(|ui| {
                                ui.label("loading boards");
                                ui.add(egui::Spinner::new());
                            });
                        } else {
                            for board in &self.board_list {
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
                        }
                    });

                ui.menu_button("settings", |ui| {
                    ui.label("lalala");
                });
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

        egui::Panel::bottom("bottom_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| match self.signalr_client.state() {
                StateWithData::Finished(_) => {
                    ui.label("connected");
                }
                StateWithData::Pending => {
                    ui.label("connecting");
                }
                StateWithData::Failed(error) => {
                    ui.label("connection error: ".to_owned() + error);
                }
                StateWithData::Idle => {
                    self.connect_signalr();
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

        if (self.selected_board != previous_board) {
            if (self.selected_board.id != 0) {
                //handle board selection
                if let StateWithData::Finished(sr_client) = self.signalr_client.state() {
                    let client = sr_client.clone();
                    let board_id = self.selected_board.id.clone();
                    println!("board {} selected", board_id);
                    self.opened_board.request(async move {
                        signalr_client_helper::test(client, board_id).await
                    });
                } else {
                    self.selected_board = IdAndNameWrapper {
                        id: 0,
                        name: "".to_owned(),
                    }
                }
            }
            if (self.selected_board.id == 0) {
                //handle board deselection
            }
        }
        if let StateWithData::Finished(data) = self.opened_board.state() {
            println!("amogus");
            println!("{}", data);
            self.opened_board.clear();
        }
    }
}

#[derive(Deserialize)]
pub struct Board {
    Id: i32,
    OwnerId: i32,
    Name: String,
    CurrentUsers: Vec<User>,
}

#[derive(Deserialize)]
pub struct User {
    Id: i32,
    Name: i32,
}
