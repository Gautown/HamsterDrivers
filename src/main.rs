// src/main.rs - 前后端分离架构入口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod gui;
mod bootstrap_icons_v1_13_1;
mod bootstrap_icons_v1_13_1_optimized;

// 调试函数：检查iconflow中可用的品牌logo图标
fn debug_available_brand_icons() {
    println!("检查iconflow中可用的品牌logo图标:");
    println!("==================================");
    
    let brand_icons = [
        "brand-asus", "brand-gigabyte", "brand-msi", "brand-intel", "brand-amd",
        "brand-dell", "brand-hp", "brand-lenovo", "brand-acer",
        "asus", "gigabyte", "msi", "intel", "amd", "dell", "hp", "lenovo", "acer",
        "computer", "desktop", "laptop", "motherboard", "circuit-board", "cpu", "factory",
        "windows-logo", "info", "hash", "gear"
    ];
    
    for icon_name in brand_icons.iter() {
        match iconflow::try_icon(iconflow::Pack::Bootstrap, icon_name, iconflow::Style::Regular, iconflow::Size::Regular) {
            Ok(icon) => {
                println!("✅ {}: 可用 (字符: {}, 字体族: {})", 
                    icon_name, 
                    char::from_u32(icon.codepoint).unwrap_or('?'),
                    icon.family);
            }
            Err(_) => {
                println!("❌ {}: 不可用", icon_name);
            }
        }
    }
}

// 调试函数：检查r-square相关图标的可用性
fn debug_r_square_icons() {
    println!("检查Bootstrap图标中r-square相关图标的可用性:");
    println!("============================================");
    
    let r_square_icons = [
        "r-square", "r-square-fill", "bi-r-square", "square-r", "square-R",
        "r", "R", "square", "square-fill",
        "registered", "copyright", "trademark"
    ];
    
    for icon_name in r_square_icons.iter() {
        match iconflow::try_icon(iconflow::Pack::Bootstrap, icon_name, iconflow::Style::Regular, iconflow::Size::Regular) {
            Ok(icon) => {
                println!("✅ {}: 可用 (字符: {}, 字体族: {})", 
                    icon_name, 
                    char::from_u32(icon.codepoint).unwrap_or('?'),
                    icon.family);
            }
            Err(_) => {
                println!("❌ {}: 不可用", icon_name);
            }
        }
    }
}

use eframe::egui;
use egui::FontFamily;
use std::path::PathBuf;


/// 安装 iconflow 字体
fn install_iconflow_fonts(ctx: &egui::Context) {
    let mut definitions = egui::FontDefinitions::default();
    
    // 安装所有 iconflow 字体变体
    for font in iconflow::fonts() {
        let family_name = font.family.to_string();
        println!("安装 iconflow 字体: {}", family_name);
        
        // 安装字体数据
        definitions.font_data.insert(
            family_name.clone(),
            egui::FontData::from_static(font.bytes).into(),
        );
        
        // 为字体族创建条目
        let family_entry = definitions
            .families
            .entry(egui::FontFamily::Name(family_name.clone().into()))
            .or_default();
        
        // 添加当前字体到字体族
        if !family_entry.contains(&family_name) {
            family_entry.insert(0, family_name.clone());
        }
        
        // 使用 iconflow 提供的实际字体族名称，不进行硬编码映射
    }

    // 安装 Bootstrap Icons v1.13.1 字体（优化版）
    bootstrap_icons_v1_13_1_optimized::install_bootstrap_icons_simple(&mut definitions);

    // 调试：打印已安装的字体族
    println!("已安装的字体族:");
    for (family, fonts) in &definitions.families {
        println!("字体族: {:?}, 包含字体: {:?}", family, fonts);
    }

    ctx.set_fonts(definitions);
    println!("成功安装 iconflow 字体和 Bootstrap Icons v1.13.1（优化版）");
}

/// 安装 Bootstrap Icons v1.13.1 字体
fn install_bootstrap_icons_v1_13_1(definitions: &mut egui::FontDefinitions) {
    println!("安装 Bootstrap Icons v1.13.1 字体...");
    
    // 读取 Bootstrap Icons v1.13.1 字体文件
    let bootstrap_font_path = "bootstrap-icons-v1.13.1/bootstrap-icons-1.13.1/fonts/bootstrap-icons.woff2";
    
    match std::fs::read(bootstrap_font_path) {
        Ok(font_bytes) => {
            // 安装 Bootstrap Icons v1.13.1 字体
            let family_name = "Bootstrap Icons v1.13.1";
            let font_id = family_name.to_string();
            
            definitions.font_data.insert(
                font_id.clone(),
                egui::FontData::from_owned(font_bytes).into(),
            );
            
            // 为字体族创建条目
            let family_entry = definitions
                .families
                .entry(egui::FontFamily::Name(family_name.into()))
                .or_default();
            
            // 添加当前字体到字体族
            if !family_entry.contains(&font_id) {
                family_entry.insert(0, font_id);
            }
            
            println!("✅ 成功安装 Bootstrap Icons v1.13.1 字体");
        }
        Err(e) => {
            println!("❌ 无法读取 Bootstrap Icons v1.13.1 字体文件: {}", e);
            println!("   字体文件路径: {}", bootstrap_font_path);
        }
    }
}

fn main() -> Result<(), eframe::Error> {


    env_logger::init();
    
    // 调试：检查iconflow中可用的品牌logo图标
    debug_available_brand_icons();
    
    // 调试：检查r-square相关图标的可用性
    debug_r_square_icons();
    
    // 调试：检查Bootstrap Icons v1.13.1中可用的图标（优化版）
    bootstrap_icons_v1_13_1_optimized::debug_bootstrap_icons_v1_13_1_optimized();
    
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
    // 首先安装 iconflow 字体
    install_iconflow_fonts(ctx);
    
    // 优先尝试动态加载系统中文字体
    match load_fonts_dynamically(ctx) {
        Ok(_) => {
            println!("成功加载系统中文字体");
        }
        Err(e) => {
            println!("动态字体加载失败: {}, 使用默认字体配置", e);
            // 回退到默认字体配置，但保留 iconflow 字体
            let mut fonts = egui::FontDefinitions::default();
            
            // 重新安装 iconflow 字体到默认配置中
            for font in iconflow::fonts() {
                fonts.font_data.insert(
                    font.family.to_string(),
                    egui::FontData::from_static(font.bytes).into(),
                );
                fonts.families
                    .entry(egui::FontFamily::Name(font.family.into()))
                    .or_default()
                    .insert(0, font.family.to_string());
            }
            
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
        
        // 获取当前的字体定义，保留已安装的 iconflow 字体
        let mut fonts = egui::FontDefinitions::default();
        
        // 重新安装 iconflow 字体
        for font in iconflow::fonts() {
            fonts.font_data.insert(
                font.family.to_string(),
                egui::FontData::from_static(font.bytes).into(),
            );
            fonts.families
                .entry(egui::FontFamily::Name(font.family.into()))
                .or_default()
                .insert(0, font.family.to_string());
        }
        
        // 添加系统中文字体
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
