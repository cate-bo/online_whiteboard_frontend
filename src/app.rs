use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use eframe::wgpu::CompilationMessageType::Info;
use egui::emath::TSTransform;
use egui::scroll_area::ScrollSource;
use egui::{
    self, ColorImage, Context, Id, Modal, Popup, TextureHandle, epaint::TextureManager, widgets,
};
use egui::{Color32, DragPanButtons, Pos2, Rect, Response, Sense, TextureOptions, Vec2, menu};
use egui::{Label, Plugin, mutex::RwLock};
use egui_async::StateWithData::Finished;
use egui_async::{Bind, EguiAsyncPlugin, StateWithData, bind::MaybeSend};
use egui_flex::{Flex, item};
use futures::future::MaybeDone;
use image::Rgba;
use image::{ImageBuffer, Pixel};
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

use crate::app::Update::Boardupdate;
use crate::frame_history::FrameHistory;
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
    frame_history: crate::frame_history::FrameHistory,
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
            frame_history: FrameHistory::default(),
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
            image::load_from_memory(&(BASE64_STANDARD.decode(&data.Drawing).unwrap())).unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let drawing = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let mut users: Vec<user> = Vec::new();
        // for user_wrapper in data.currentEditors {}
        let mut images: Vec<Image> = Vec::new();
        for image_wrapper in data.Images {
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
            if (login_info.id == data.OwnerId) {
                permission = BoardPermission::Owner;
            } else {
                for user in &data.CurrentEditors {
                    if (login_info.id == user.id) {
                        permission = BoardPermission::Editor;
                        break;
                    }
                }
            }
        }

        let (draw_sender, reciever) = mpsc::channel::<Vec<DrawUpdate>>();
        let drawing_buffer = drawing.clone();
        let th = context.load_texture("drawing", drawing.clone(), Default::default());
        let th2 = th.clone();
        //spawn(async move { apply_draw_updates(reciever, th, drawing_buffer, size).await });

        let board = Whiteboard {
            id: data.Id,
            ownerId: data.OwnerId,
            name: data.Name,
            drawing_texture: th2,
            drawing_buffer: drawing_buffer,
            size: size,
            currentEditors: data.CurrentEditors,
            texts: data.Texts,
            images: images,
            permission: permission,
        };
        println!("amogus");
        sender.send(Update::Boardloaded(board));
    }

    fn apply_board_update(&mut self, update: BoardUpdate) {
        if let Some(board) = &mut self.current_whiteboard {
            for draw_update in update.draw_updates {
                for (x, y) in draw_update.coords {
                    board.drawing_buffer.pixels[(y as usize * board.size[0] + x) as usize] =
                        draw_update.color;
                }
            }
            board
                .drawing_texture
                .set(board.drawing_buffer.clone(), TextureOptions::NEAREST);
            // board.drawing_texture.set(
            //     egui::ColorImage::new(board.size, board.drawing_buffer.clone()),
            //     TextureOptions::NEAREST,
            // );
        }
    }

    fn create_board_update(&mut self, action: Action) {
        match action {
            Action::Draw(draw_update) => {
                let board_update = BoardUpdate {
                    draw_updates: vec![draw_update],
                };
                self.apply_board_update(board_update);
                // send update to server here
            }
            _ => {}
        }
    }

    // fn apply_draw_updates(&mut self, updates: Vec<DrawUpdate>) {
    //     // let pixels = drawing_buffer.as_flat_samples();
    //     // let size = [5000_usize, 5000_usize];
    //     // let drawing = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    //     th.set(
    //         egui::ColorImage::new(size, drawing_buffer.clone()),
    //         TextureOptions::NEAREST,
    //     );
    // }
}

impl eframe::App for WhiteboardApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
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
        if self.selected_board.id == 0 {
            self.board_update_queue.clear();
        }
        if let Some(Whiteboard) = &mut self.current_whiteboard {
            while let Some(board_update) = self.board_update_queue.pop_front() {
                self.apply_board_update(board_update);
            }
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
            ui.horizontal(|ui| {
                ui.label(format!("fps: {:.1}", self.frame_history.fps()));
                ui.label("");
                match self.signalr_client.state() {
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
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(whiteboard) = &mut self.current_whiteboard {
                egui::Panel::top("board_menu").show(ui, |ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for editor in &whiteboard.currentEditors {
                                ui.label(&editor.name);
                            }
                        });
                    });
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
                let mut action: Action = Action::None;

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
                                        let mut res = ui.add(drawing);
                                        match self.selected_tool {
                                            Tool::Navigate => {
                                                res.on_hover_cursor(egui::CursorIcon::AllScroll);
                                            }
                                            Tool::Brush => {
                                                res =
                                                    res.on_hover_cursor(egui::CursorIcon::Default);
                                                if res.dragged() {
                                                    let pos = ui
                                                        .ctx()
                                                        .layer_transform_from_global(
                                                            ui.painter().layer_id(),
                                                        )
                                                        .unwrap_or_default()
                                                        * ui.input(|i| {
                                                            i.pointer
                                                                .interact_pos()
                                                                .unwrap_or_default()
                                                        });
                                                    // if let Some(pos) =
                                                    //     ui.ctx().input(|i| i.pointer.interact_pos())
                                                    // {
                                                    // }
                                                    // println!("{}", pos);

                                                    let delta = res.drag_delta();
                                                    // println!("{}", delta);
                                                    let brush_size = self.brush_size.clone();
                                                    // let sender = self.sender.clone();
                                                    let color = self.current_color.clone();
                                                    action = Action::Draw(create_draw_update(
                                                        brush_size, pos, color, delta,
                                                    ));
                                                }
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

                self.create_board_update(action);

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
                    flex.add(item().grow(1_f32), Label::new(""));
                    flex.add(item(), Label::new("loading"));
                    flex.add(item(), egui::Spinner::new());
                    flex.add(item().grow(1_f32), Label::new(""));
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
//#[serde(rename_all = "lowercase")]
pub struct OpenWhiteboardResponse {
    Id: i32,
    OwnerId: i32,
    Name: String,
    Drawing: String,
    CurrentEditors: Vec<user>,
    Texts: Vec<Text>,
    Images: Vec<ImageWrapper>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub struct user {
    id: i32,
    name: String,
}

impl PartialEq for user {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
    drawing_buffer: ColorImage,
    size: [usize; 2],
    currentEditors: Vec<user>,
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
    draw_updates: Vec<DrawUpdate>,
}

struct DrawUpdate {
    color: Color32,
    coords: Vec<(usize, usize)>,
}

#[derive(PartialEq, Clone)]
enum Tool {
    Brush,
    Navigate,
}

enum BoardPermission {
    Owner,
    Editor,
    Viewer,
}

enum Action {
    Draw(DrawUpdate),
    None,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + MaybeSend + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::spawn(future);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);
}

fn create_draw_update(brush_size: i32, pos: Pos2, color: Color32, drag_delta: Vec2) -> DrawUpdate {
    let mut coords: Vec<(usize, usize)> = Vec::new();
    let pos_x = pos.x.round() as i32;
    let pos_y = pos.y.round() as i32;
    let delta_x = pos_x - drag_delta.x as i32;
    let delta_y = pos_y - drag_delta.y as i32;
    let x_distance = drag_delta.x.abs();
    let y_distance = drag_delta.y.abs();
    let mut min_x = -brush_size;
    let mut max_x = brush_size;
    let mut x_to_y_ratio = delta_x / pos_y;
    if delta_x < pos_x {
        min_x += delta_x;
        max_x += pos_x;
        x_to_y_ratio = pos_x / delta_y;
    } else {
        min_x += pos_x;
        max_x += delta_x;
    }
    let smaller_x = min_x + brush_size;
    if min_x < 0 {
        min_x = 0;
    }
    if max_x > 5000 {
        max_x = 5000;
    }
    //println!("{},{}", min_x, max_x);
    let mut min_y = -brush_size;
    let mut max_y = brush_size;
    let mut y_to_x_ratio = delta_y / pos_x;
    if delta_y < pos_y {
        min_y += delta_y;
        max_y += pos_y;
        y_to_x_ratio = pos_y / delta_x;
    } else {
        min_y += pos_y;
        max_y += delta_y;
    }
    let smaller_y = min_y + brush_size;
    if min_y < 0 {
        min_y = 0;
    }
    if max_y > 5000 {
        max_y = 5000;
    }
    let threshold = brush_size * brush_size;
    for x in min_x..=max_x {
        // if x < 0 || x > 5000 {
        //     continue;
        // }
        let pos_x_distance = (x - pos_x).abs();
        let delta_x_distance = (x - delta_x).abs();
        let pos_x_offset = pos_x_distance * pos_x_distance;
        let delta_x_offset = delta_x_distance * delta_x_distance;
        let x_difference = (pos_x_distance - delta_x_distance).abs();
        let x_offset = x_difference * x_difference;
        let x_between = min_x + brush_size < x && x < max_x - brush_size;
        let to_smaller_x = (x - smaller_x) as f32;
        for y in min_y..=max_y {
            // if y < 0 || y > 5000 {
            //     continue;
            // }
            let pos_y_distance = (y - pos_y).abs();
            let delta_y_distance = (y - delta_y).abs();
            let pos_y_offset = pos_y_distance * pos_y_distance;
            let delta_y_offset = delta_y_distance * delta_y_distance;
            let y_difference = (pos_y_distance - delta_y_distance).abs();
            let y_offset = y_difference * y_difference;
            let pos_offset = pos_x_offset + pos_y_offset;
            let delta_offset = delta_x_offset + delta_y_offset;
            let pos_average = (pos_x_distance + pos_y_distance) / 2;
            let delta_average = (delta_x_distance + delta_y_distance) / 2;
            let pos_ratio = pos_x / pos_y;
            let delta_ratio = delta_x / delta_y;
            let ratio_difference = (pos_ratio - delta_ratio).abs();
            let ratio_average = (pos_ratio + delta_ratio) / 2;
            let xy_ratio = x as f32 / y as f32;
            let yx_ratio = y as f32 / x as f32;
            let y_between = min_y + brush_size < y && y < max_y - brush_size;
            let to_smaller_y = (y - smaller_y) as f32;
            let in_range = ((x_distance / y_distance).abs() - (to_smaller_x / to_smaller_y).abs()).abs()
                < 1_f32
                && ((y_distance / x_distance).abs() - (to_smaller_y / to_smaller_x).abs()).abs() < 1_f32;

            if (pos_offset < threshold
                || delta_offset < threshold
                || (x_between && y_between && in_range))
            {
                coords.push((x as usize, y as usize));
            }
        }
    }
    DrawUpdate {
        color: color,
        coords: coords,
    }
}

// async fn apply_draw_updates(
//     reciever: Receiver<Vec<DrawUpdate>>,
//     mut th: TextureHandle,
//     mut drawing_buffer: Vec<egui::Color32>,
//     size: [usize; 2],
// ) {
//     while let Ok(msg) = reciever.recv() {
//         for draw_update in msg {
//             for (x, y) in draw_update.coords {
//                 drawing_buffer[(y as usize * size[0] + x) as usize] = draw_update.color;
//             }
//         }

//         // let pixels = drawing_buffer.as_flat_samples();
//         // let size = [5000_usize, 5000_usize];
//         // let drawing = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
//         th.set(
//             egui::ColorImage::new(size, drawing_buffer.clone()),
//             TextureOptions::NEAREST,
//         );
//     }
// // }

// fn create_draw_update(pos: Pos2, color: Color32, brush_size: i32) -> DrawUpdate {
//     let mut coords: Vec<(usize, usize)> = Vec::new();
//     let posx = pos.x.round() as i32;
//     let posy = pos.y.round() as i32;
//     let threshold = brush_size * brush_size;
//     for x in posx - brush_size..posx + brush_size {
//         if x < 0 || x > 5000 {
//             continue;
//         }
//         let x_distance = (x - posx).abs();
//         let x_offset = x_distance * x_distance;
//         for y in posy - brush_size..posy + brush_size {
//             if y < 0 || y > 5000 {
//                 continue;
//             }
//             let y_distance = (y - posy).abs();
//             let y_offset = y_distance * y_distance;
//             if (x_offset + y_offset) < threshold {
//                 coords.push((x as usize, y as usize));
//             }
//         }
//     }
//     DrawUpdate {
//         color: color,
//         coords: coords,
//     }
// }
