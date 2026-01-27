// src/main.rs - 前后端分离架构入口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod gui;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {


    env_logger::init();
    let icon_data = load_icon("assets/icons/icon.ico");
    let viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1024.0, 668.0])
        .with_min_inner_size([600.0, 400.0])
        .with_position(egui::Pos2 { x: (1920.0 - 1024.0) / 2.0, y: (1080.0 - 668.0) / 2.0 })
        .with_decorations(false)
        .with_resizable(false)
        .with_transparent(true);

    let viewport_builder = if let Some(icon) = icon_data {
        viewport_builder.with_icon(icon)
    } else {
        viewport_builder
    };
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };
    eframe::run_native(
        "Hamster Drivers Manager",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            populate_chinese_fonts(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);

            let app = gui::GuiApp::new().expect("Failed to initialize app state");
            Ok::<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>(Box::new(app))
        })
    )
}

fn populate_chinese_fonts(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert(
        "chinese_font".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/SourceHanSerifSC-Regular.otf"))),
    );

    use egui::FontFamily::{Proportional, Monospace};
    let chinese_font = "chinese_font".to_owned();
    let proportional = fonts.families.entry(Proportional).or_default();
    if !proportional.contains(&chinese_font) {
        proportional.insert(0, chinese_font.clone());
    }
    let monospace = fonts.families.entry(Monospace).or_default();
    if !monospace.contains(&chinese_font) {
        monospace.insert(0, chinese_font);
    }
}

fn load_icon(path: &str) -> Option<egui::IconData> {
    let path = std::path::Path::new(path);
    match std::fs::read(path) {
        Ok(data) => {
            match image::load_from_memory(&data) {
                Ok(img) => {
                    let image = img.to_rgba8();
                    let (width, height) = image.dimensions();
                    let rgba: Vec<u8> = image.into_raw();
                    Some(egui::IconData {
                        rgba,
                        width,
                        height,
                    })
                }
                Err(e) => {
                    eprintln!("Failed to decode image: {e}");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read icon file: {e}");
            None
        }
    }
}
