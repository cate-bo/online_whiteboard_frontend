use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use eframe::wgpu::CompilationMessageType::Info;
use egui::emath::TSTransform;
use egui::scroll_area::ScrollSource;
use egui::{
    self, ColorImage, Context, Id, Modal, Popup, TextureHandle, epaint::TextureManager, widgets,
};
use egui::{Color32, DragPanButtons, Pos2, Rect, Response, Sense, menu};
use egui::{Label, Plugin, mutex::RwLock};
use egui_async::StateWithData::Finished;
use egui_async::{Bind, EguiAsyncPlugin, StateWithData, bind::MaybeSend};
use egui_flex::{Flex, item};
use futures::future::MaybeDone;
use image;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{self, Value};
use signalr_client::SignalRClient;
use std::{
    borrow::Cow,
    collections::VecDeque,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
};

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
    test: Bind<Value, String>,
    current_whiteboard: Option<Whiteboard>,
    reciever: Receiver<Update>,
    sender: Sender<Update>,
    board_update_queue: VecDeque<BoardUpdate>,
    selected_tool: Tool,
    scene_rect: Rect,
    current_color: Color32,
    brush_size: i32,
}

impl WhiteboardApp {
    pub fn new(c: &eframe::CreationContext<'_>) -> Self {
        let (send, recv) = mpsc::channel::<Update>();
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
            test: Bind::new(true),
            current_whiteboard: None,
            reciever: recv,
            sender: send,
            board_update_queue: VecDeque::new(),
            selected_tool: Tool::Navigate,
            scene_rect: Rect::from_pos(Pos2::new(0_f32, 0_f32)),
            current_color: Color32::BLACK,
            brush_size: 5,
        };
        temp.refresh_boards();
        temp.connect_signalr();
        return temp;
    }

    fn login_changed(&mut self) {
        self.refresh_boards();
        self.connect_signalr();

        self.selected_board = IdAndNameWrapper {
            id: 0,
            name: "".to_owned(),
        };
        self.current_whiteboard = None;
    }

    fn connect_signalr(&mut self) {
        if let StateWithData::Finished(client) = self.signalr_client.state() {
            client.clone().disconnect();
        }
        println!("trying to connect to signalr");
        let mut info: Option<LoginInfo> = None;
        if let StateWithData::Finished(login_info) = self.login.state() {
            info = Some(login_info.clone());
        }
        let sender = self.sender.clone();
        self.signalr_client.clear();
        self.signalr_client
            .request(async move { signalr_client_helper::connect(info, sender).await })
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

    fn load_whiteboard(
        data: OpenWhiteboardResponse,
        context: Context,
        login: Option<LoginInfo>,
        sender: Sender<Update>,
    ) {
        let image =
            image::load_from_memory(&(BASE64_STANDARD.decode(&data.drawing).unwrap())).unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let drawing = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let mut images: Vec<Image> = Vec::new();
        for image_wrapper in data.images {
            let image = image::load_from_memory_with_format(
                &image_wrapper.File.as_bytes(),
                image::ImageFormat::Png,
            )
            .unwrap();
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            images.push(Image {
                id: image_wrapper.Id,
                x: image_wrapper.X,
                y: image_wrapper.Y,
                file: context.load_texture(
                    format!("image_{}", image_wrapper.Id),
                    image,
                    Default::default(),
                ),
            });
        }
        let mut permission = BoardPermission::Viewer;
        if let Some(login_info) = login {
            if (login_info.id == data.ownerId) {
                permission = BoardPermission::Owner;
            } else {
                for user in &data.currentEditors {
                    if (login_info.id == user.Id) {
                        permission = BoardPermission::Editor;
                        break;
                    }
                }
            }
        }

        let board = Whiteboard {
            id: data.id,
            ownerId: data.ownerId,
            name: data.name,
            drawing_texture: context.load_texture("drawing", drawing.clone(), Default::default()),
            drawing_Image: drawing,
            currentEditors: data.currentEditors,
            texts: data.texts,
            images: images,
            permission: permission,
        };
        println!("amogus");
        sender.send(Update::Boardloaded(board));
    }

    fn apply_board_update(&mut self, update: BoardUpdate) {
        //TODO
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
        }
        self.previously_logged_in = currently_logged_in;
        for update in self.reciever.try_iter() {
            match update {
                Update::Boardupdate(boardupdate) => {
                    self.board_update_queue.push_back(boardupdate);
                }
                Update::Boardrecieved(boardresponse) => {
                    self.board_update_queue.clear();
                    let sender = self.sender.clone();
                    let context = ctx.clone();
                    let mut login: Option<LoginInfo> = None;
                    if let StateWithData::Finished(info) = self.login.state() {
                        login = Some(info.clone());
                    }
                    spawn(async move {
                        WhiteboardApp::load_whiteboard(boardresponse, context, login, sender);
                    });
                }
                Update::Boardloaded(board) => {
                    self.current_whiteboard = Some(board);
                }
                Update::BoardError => {
                    self.selected_board = IdAndNameWrapper {
                        id: 0,
                        name: "".to_owned(),
                    };
                    self.current_whiteboard = None;
                }
            }
        }

        if let Some(Whiteboard) = &mut self.current_whiteboard {
            while let Some(BoardUpdate) = self.board_update_queue.pop_front() {}
        }

        ctx.request_repaint();
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
                ui.menu_button("tests", |ui| {
                    if ui.button("test1").clicked() {
                        if let StateWithData::Finished(client) = self.signalr_client.state() {
                            let c = client.clone();
                            self.test
                                .request(async move { signalr_client_helper::test(c).await })
                        }
                    }
                });
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

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(whiteboard) = &self.current_whiteboard {
                egui::Panel::top("board_menu").show(ui, |ui| {
                    if let BoardPermission::Viewer = whiteboard.permission {
                    } else {
                        menu::MenuBar::new().ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_tool,
                                Tool::Navigate,
                                "navigate",
                            );
                            ui.selectable_value(&mut self.selected_tool, Tool::Brush, "brush");
                            ui.color_edit_button_srgba(&mut self.current_color);
                            ui.add(egui::DragValue::new(&mut self.brush_size).range(1..=50));
                        });
                    }
                });

                egui::CentralPanel::default()
                    // .frame(egui::Frame::new().fill(Color32::WHITE))
                    .show(ui, |ui| {
                        let mut scene = egui::Scene::new().drag_pan_buttons(DragPanButtons::MIDDLE);
                        if let Tool::Navigate = self.selected_tool {
                            scene = scene.drag_pan_buttons(DragPanButtons::all());
                        }
                        let mut response = scene
                            .show(ui, &mut self.scene_rect, |ui| {
                                egui::Frame::NONE
                                    .fill(Color32::WHITE)
                                    .show(ui, |ui| {
                                        let size = whiteboard.drawing_texture.size_vec2();
                                        let sized_texture = egui::load::SizedTexture::new(
                                            whiteboard.drawing_texture.id(),
                                            size,
                                        );
                                        let mut drawing =
                                            egui::Image::new(sized_texture).fit_to_exact_size(size);
                                        if let Tool::Navigate = self.selected_tool {
                                            drawing = drawing.sense(Sense::empty());
                                        } else {
                                            drawing = drawing.sense(Sense::drag());
                                        }
                                        let res = ui.add(drawing);
                                        match self.selected_tool {
                                            Tool::Navigate => {
                                                res.on_hover_cursor(egui::CursorIcon::AllScroll);
                                            }
                                            Tool::Brush => {
                                                res.on_hover_cursor(egui::CursorIcon::Default);
                                            }
                                            _ => {}
                                        }
                                    })
                                    .response;
                            })
                            .response;
                        response.on_hover_and_drag_cursor(egui::CursorIcon::AllScroll);
                        // if self.scene_rect.top() < 0_f32 {
                        //     self.scene_rect.set_top(0_f32);
                        // }
                        // if self.scene_rect.bottom() > 5000_f32 {
                        //     self.scene_rect.set_bottom(5000_f32);
                        // }
                        // if self.scene_rect.left() < 0_f32 {
                        //     self.scene_rect.set_left(0_f32);
                        // }
                        // if self.scene_rect.right() > 5000_f32 {
                        //     self.scene_rect.set_right(5000_f32);
                        // }

                        // let mut thing = TSTransform::default();
                        // scene.register_pan_and_zoom(ui, &mut response, &mut thing);
                        // response.
                        // println!("{:?}", response);
                        // println!("{:?}", thing);
                        // if self.scene_rect.top() < 0_f32 {
                        //     self.scene_rect.set_top(0_f32);
                        // }
                        // let size = whiteboard.drawing.size_vec2();
                        // let sized_texture =
                        //     egui::load::SizedTexture::new(whiteboard.drawing.id(), size);
                        // ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
                    });

                // egui::Frame::new().fill(Color32::GREEN).show(ui, |ui| {});
                // let size = whiteboard.drawing.size_vec2();
                // let sized_texture = egui::load::SizedTexture::new(whiteboard.drawing.id(), size);
                // ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
                // Flex::vertical().h_full().w_full().show(ui, |flex| {
                //     flex.add_ui(item(), |ui| {});
                // });
            } else if self.selected_board.id != 0
                && let None = self.current_whiteboard
            {
                Flex::new().h_full().w_full().show(ui, |flex| {
                    flex.add(item().grow(1_f32), Label::new("loading"));
                    flex.add(item(), egui::Spinner::new());
                });
            } else {
                Flex::new().h_full().w_full().show(ui, |flex| {
                    flex.add(item().grow(1_f32), Label::new("no board selected"));
                });
            }
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
            self.current_whiteboard = None;
            if (self.selected_board.id != 0) {
                //handle board selection
                if let StateWithData::Finished(sr_client) = self.signalr_client.state() {
                    let client = sr_client.clone();
                    let board_id = self.selected_board.id.clone();
                    let sender = self.sender.clone();
                    println!("board {} selected", board_id);
                    spawn(async move {
                        signalr_client_helper::open_whiteboard(client, board_id, sender).await
                    });
                } else {
                    self.selected_board = IdAndNameWrapper {
                        id: 0,
                        name: "".to_owned(),
                    }
                }
            } else {
                //handle board deselection
            }
        }

        if let StateWithData::Finished(result) = self.test.state() {
            //println!("test2: {}", serde_json::to_string_pretty(result).unwrap());
            println!("amogus: {:?}", result);
            println!();
            //let thing: Board = serde_json::from_str(result).unwrap();
            let thing: OpenWhiteboardResponse = serde_json::from_value(result.clone()).unwrap();
            println!("amogus: {:?}", thing);
            self.test.clear();
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenWhiteboardResponse {
    id: i32,
    ownerId: i32,
    name: String,
    drawing: String,
    currentEditors: Vec<User>,
    texts: Vec<Text>,
    images: Vec<ImageWrapper>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    Id: i32,
    Name: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Text {
    Id: i32,
    X: i32,
    Y: i32,
    Text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ImageWrapper {
    Id: i32,
    X: i32,
    Y: i32,
    File: String,
}

pub struct Whiteboard {
    id: i32,
    ownerId: i32,
    name: String,
    drawing_texture: TextureHandle,
    drawing_Image: ColorImage,
    currentEditors: Vec<User>,
    texts: Vec<Text>,
    images: Vec<Image>,
    permission: BoardPermission,
}

pub struct Image {
    id: i32,
    x: i32,
    y: i32,
    file: TextureHandle,
}

pub enum Update {
    Boardrecieved(OpenWhiteboardResponse),
    BoardError,
    Boardloaded(Whiteboard),
    Boardupdate(BoardUpdate),
}

pub struct BoardUpdate {
    //TODO
}

#[derive(PartialEq)]
enum Tool {
    Brush,
    Navigate,
}

enum BoardPermission {
    Owner,
    Editor,
    Viewer,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::spawn(future);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);
}
