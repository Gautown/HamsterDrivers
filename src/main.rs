// src/main.rs - 前后端分离架构入口
mod Core;
mod gui;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // 初始化日志
    
    // 加载图标
    let icon_data = load_icon("assets/icons/icon.ico");
    
    let viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1024.0, 668.0])
        .with_min_inner_size([600.0, 400.0])
        .with_position(egui::Pos2 { x: (1920.0 - 1024.0) / 2.0, y: (1080.0 - 668.0) / 2.0 }) // 居中位置（基于1920x1080屏幕分辨率计算）
        .with_decorations(false)  // 去掉标题栏
        .with_resizable(false)    // 禁止调整大小
        .with_transparent(false);  // 关闭透明，使用默认窗口样式以支持系统阴影
    
    // 如果加载图标成功，则设置图标
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
            // 设置中文字体
            let mut fonts = egui::FontDefinitions::default();
            populate_chinese_fonts(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);
            
            // 创建GUI应用实例
            let app = gui::GuiApp::new().expect("Failed to initialize app state");
            
            Ok(Box::new(app))
        })
    )
}

fn populate_chinese_fonts(fonts: &mut egui::FontDefinitions) {
    // 添加中文字体支持
    fonts.font_data.insert(
        "chinese_font".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/SourceHanSerifSC-Regular.otf"))),
    );
    
    // 配置字体族使用中文字体
    use egui::FontFamily::{Proportional, Monospace};
    if let Some(proportional) = fonts.families.get_mut(&Proportional) {
        proportional.insert(0, "chinese_font".to_owned());
    }
    if let Some(monospace) = fonts.families.get_mut(&Monospace) {
        monospace.insert(0, "chinese_font".to_owned());
    }
}

fn load_icon(path: &str) -> Option<egui::IconData> {
    // 尝试从文件加载图标
    match std::fs::read(std::path::Path::new(path)) {
        Ok(data) => {
            // 解码图像数据
            match image::load_from_memory(&data) {
                Ok(image) => {
                    let image = image.to_rgba8();
                    let (width, height) = image.dimensions();
                    let rgba = image.as_raw().clone();
                    
                    Some(egui::IconData {
                        rgba,
                        width,
                        height,
                    })
                }
                Err(e) => {
                    eprintln!("Failed to decode image: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read icon file: {}", e);
            None
        }
    }
}