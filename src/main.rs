// src/main.rs - 前后端分离架构入口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod gui;

use eframe::egui;
use egui::FontFamily;
use std::path::PathBuf;

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
            // 配置egui使用系统字体
            configure_system_fonts(&cc.egui_ctx);
            
            let app = gui::GuiApp::new().expect("Failed to initialize app state");
            Ok::<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>(Box::new(app))
        }),
    )
}

/// 配置egui字体设置
fn configure_system_fonts(ctx: &egui::Context) {
    // 优先尝试动态加载系统中文字体
    match load_fonts_dynamically(ctx) {
        Ok(_) => {
            println!("成功加载系统中文字体");
        }
        Err(e) => {
            println!("动态字体加载失败: {}, 使用默认字体配置", e);
            // 回退到默认字体配置
            let fonts = egui::FontDefinitions::default();
            ctx.set_fonts(fonts);
        }
    }
    
    // 设置字体大小和样式
    let mut style = (*ctx.style()).clone();
    
    // 配置文本样式，确保中文显示清晰
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(20.0, FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(14.0, FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(12.0, FontFamily::Monospace)),
        (egui::TextStyle::Button, egui::FontId::new(14.0, FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(10.0, FontFamily::Proportional)),
    ]
    .into();
    
    // 配置UI缩放，确保在不同DPI设置下中文显示正常
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 248, 248);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(240, 240, 240);
    
    // 应用样式配置
    ctx.set_style(style);
    
    // 设置像素完美渲染，确保字体清晰
    ctx.set_pixels_per_point(1.0);
}

fn find_system_font_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Windows 常见字体路径
        let paths = vec![
            r"C:\Windows\Fonts\simsun.ttc", // 宋体
            r"C:\Windows\Fonts\msyh.ttc",   // 微软雅黑
            r"C:\Windows\Fonts\simhei.ttf", // 黑体
            r"C:\Windows\Fonts\simkai.ttf", // 楷体
        ];
        paths.into_iter().find(|p| std::path::Path::new(p).exists()).map(PathBuf::from)
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 字体路径
        let paths = vec![
            "/System/Library/Fonts/PingFang.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ];
        paths.into_iter().find(|p| std::path::Path::new(p).exists()).map(PathBuf::from)
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 字体路径 (更分散)
        use std::process::Command;
        // 尝试通过 `fc-match` 命令查找中文字体
        if let Ok(output) = Command::new("fc-match").args(&["-f", "%{file}", "serif:lang=zh"]).output() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
        None
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    None
}

fn load_fonts_dynamically(ctx: &egui::Context) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(font_path) = find_system_font_path() {
        println!("找到系统中文字体: {:?}", font_path);
        let font_data = std::fs::read(font_path)?; // 读取字体文件
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("system_chinese".to_owned(), egui::FontData::from_owned(font_data).into());
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system_chinese".to_owned());
        ctx.set_fonts(fonts);
        Ok(())
    } else {
        println!("未找到可用的系统中文字体，使用默认字体");
        Err("未找到可用的系统中文字体".into())
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
