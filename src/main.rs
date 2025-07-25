#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, Mutex},
    thread
};

use eframe::{
    egui,
    epaint::text::{FontInsert, InsertFontFamily},
};

mod find_id;
use crate::find_id::SearchMode;


fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui example: custom font",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

fn add_font(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "my_font",
        egui::FontData::from_static(include_bytes!("../assets/simkai.ttf")),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
}

struct App {
    query: String,
    path: String,
    mode: find_id::SearchMode,
    results: Arc<Mutex<Vec<find_id::MatchInfo>>>,
    msg: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        add_font(&cc.egui_ctx);
        Self {
            query: "".to_string(),
            path: "".to_string(),
            mode: find_id::SearchMode::Guid,
            results: Arc::new(Mutex::new(Vec::new())),
            msg: "".to_string(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Find ID");

            // 设置控件之间的默认间距
            let spacing = ui.spacing_mut();
            spacing.item_spacing = egui::vec2(40.0, 20.0);

            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut self.path);
                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.path = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.radio_value(&mut self.mode, SearchMode::MediaID, "MediaID");
                ui.radio_value(&mut self.mode, SearchMode::Guid, "Guid");
                ui.radio_value(&mut self.mode, SearchMode::ShortID, "ShortID");
            });

            // 输入框
            ui.horizontal(|ui|{
                ui.label("Query:");
                ui.text_edit_singleline(&mut self.query);
            });

            ui.label(format!("Path: {}", self.path));

            if ui.button("Search").clicked() {
                let query = self.query.clone();
                let path = self.path.clone();
                if !find_id::is_path_valid(&path).is_ok() {
                    self.msg = format!("Invalid path: {}", path);
                    ctx.request_repaint();
                    return;
                }
                let mode = self.mode.clone();
                let results = Arc::clone(&self.results);
                thread::spawn(move || {
                    println!("Searching for {} in {}", query, path);
                    let res = find_id::find_id(&query, &path, &mode);
                    println!("Results: {:?}", res.len());
                    let mut results = results.lock().unwrap();
                    *results = res;
                });
            }

            let results = self.results.lock().unwrap();
            if results.is_empty() {
                ui.label("No results found.");
            } else {
                ui.label("Results:");
                for result in results.iter() {
                    ui.label(format!("{:?}", result));
                }
            }

            // 显示消息
            ui.label(format!("{}", self.msg));
        });
    }
}
