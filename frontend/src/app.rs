use std::process::id;

use eframe::{egui, glow::NONE};
use egui::{LayerId, Response};

enum Tab {
    Zombies,
    Logs,
    Settings,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    tab: Tab,

    #[serde(skip)]
    selected_row: Option<usize>,

    #[serde(skip)]
    context_menu_id: Option<usize>,

    #[serde(skip)]
    response: Option<Response>,

    backend_url: String,
    username: String,
    password: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::Zombies,
            selected_row: None,
            context_menu_id: None,
            response: None,
            backend_url: "https://localhost:3000/api".to_owned(),
            username: "User".to_owned(),
            password: "123456".to_owned(),
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
    fn selectable_row(&mut self, ui: &mut egui::Ui, index: usize, row: &Vec<&str>) {
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
       ui.label(egui::RichText::new("STATUS").color(egui::Color32::LIGHT_BLUE).text_style(egui::TextStyle::Button));
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

               //first user
               for i in 0..3 {
                    let row = vec![ "192.168.2.112", "-", "android", "erarnitox", "Linux", "ONLINE"];
                    app.selectable_row(ui, i, &row);
                    
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

    if ui.button("Save & Connect!").clicked() {
        //TODO: authentication retrival of Auth-Token
        //TODO: save creds in the database
        //TODO: if successful hide creds in the input fields
    }
}

fn logs_ui(ui: &mut egui::Ui) {
}