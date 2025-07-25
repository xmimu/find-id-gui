#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use eframe::{
    egui,
    epaint::text::{FontInsert, InsertFontFamily},
};

mod find_id;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
    query: Arc<Mutex<String>>,
    path: Arc<Mutex<String>>,
    mode: Arc<Mutex<find_id::SearchMode>>,
    results: Arc<Mutex<Vec<find_id::MatchInfo>>>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        add_font(&cc.egui_ctx);
        Self {
            query: Arc::new(Mutex::new("".to_string())),
            path: Arc::new(Mutex::new("".to_string())),
            mode: Arc::new(Mutex::new(find_id::SearchMode::Guid)),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            
            if ui.button("Search").clicked() {
                let query = Arc::clone(&self.query);
                let path = Arc::clone(&self.path);
                let mode = Arc::clone(&self.mode);
                let results = Arc::clone(&self.results);
                thread::spawn(move || {
                    let query = query.lock().unwrap();
                    let path = path.lock().unwrap();
                    let mode = mode.lock().unwrap();
                    let mut results = results.lock().unwrap();
                    println!("Searching for {} in {}", query, path);
                    *results = find_id::find_id(&query, &path, &mode);
                    
                });
            }

            // 显示搜索结果
            let results = self.results.lock().unwrap();
            if results.is_empty() {
                ui.label("No results found.");
            } else {
                ui.label("Results:");
                for result in results.iter() {
                    ui.label(format!("{}", result.tag));
                    break;
                }
            }
            
        });
    }
}
