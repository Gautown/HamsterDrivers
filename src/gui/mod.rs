// 将原来的ui.rs内容整合进来
use eframe::egui;
use crate::core::sysinfo::SystemInfo;
// 新增导入 image crate
use webbrowser;
// SVG支持
use resvg::usvg::{self, TreeParsing};
use tiny_skia::{Pixmap, Transform};
use crate::core::features::driver_installer::{DriverInstaller, InstallableDriver};
use crate::core::features::driver_manager::DriverManagement;

use std::sync::mpsc;
use std::thread;
#[allow(dead_code)]
pub struct GuiApp {
    // 使用core模块中的类型
    pub driver_service: crate::core::windows_api::driver_service::DriverService,
    pub dependency_analyzer: crate::core::features::dependency_analyzer::DependencyAnalyzer,
    pub signature_validator: crate::core::features::signature_validator::SignatureValidator,
    pub backup_manager: crate::core::features::backup_manager::BackupManager,
    pub driver_installer: DriverInstaller,
    pub driver_management: DriverManagement,
    selected_tab: AppTab,
    pub drivers: Vec<crate::core::driver_manager::DriverInfo>,
    pub backup_history: Vec<String>,
    pub scan_in_progress: bool,
    pub selected_driver: Option<usize>,
    pub system_info: Option<SystemInfo>,
    system_info_loading: bool,
    system_info_error: Option<String>,
    system_info_rx: Option<std::sync::mpsc::Receiver<Result<SystemInfo, String>>>,
    title_icon: Option<egui::TextureHandle>,
    github_icon: Option<egui::TextureHandle>,
    // 驱动安装相关状态
    scanned_drivers: Vec<InstallableDriver>,
    selected_install_driver: Option<usize>,
    scan_directory: String,
    // 驱动管理相关状态
    driver_management_subtab: DriverManagementSubTab,
    selected_backup_file: Option<usize>,
    driver_management_expanded: bool,
}

#[derive(PartialEq)]
enum AppTab {
    Overview,
    DriverInstall,
    DriverManagement,
    DriverBackup,
    DriverRestore,
    DriverUninstall,
    SystemGameComponents,
    BackupRestore,
    Settings,
}

#[derive(PartialEq)]
enum DriverManagementSubTab {
    Backup,
}

impl GuiApp {
    pub fn new() -> Result<Self, String> {
        // 创建通道用于异步获取系统信息
        let (tx, rx) = mpsc::channel();
        
        // 在后台线程中获取系统信息
        thread::spawn(move || {
            let result = SystemInfo::new();
            let _ = tx.send(result);
        });
        
        Ok(Self {
            driver_service: crate::core::windows_api::driver_service::DriverService::new()?,
            dependency_analyzer: crate::core::features::dependency_analyzer::DependencyAnalyzer::new(),
            signature_validator: crate::core::features::signature_validator::SignatureValidator::new(),
            backup_manager: crate::core::features::backup_manager::BackupManager::new()?,
            driver_installer: DriverInstaller::new(),
            driver_management: DriverManagement::new(),
            selected_tab: AppTab::Overview,
            drivers: Vec::new(),
            backup_history: Vec::new(),
            scan_in_progress: false,
            selected_driver: None,
            system_info: None,
            system_info_loading: true,  // 开始时设为true，表示正在加载
            system_info_error: None,
            system_info_rx: Some(rx),
            title_icon: None,
            github_icon: None,
            // 驱动安装相关状态
            scanned_drivers: Vec::new(),
            selected_install_driver: None,
            scan_directory: "./".to_string(),
            // 驱动管理相关状态
            driver_management_subtab: DriverManagementSubTab::Backup,
            selected_backup_file: None,
            driver_management_expanded: false,
            // window drag handled natively on Windows
        })
    }

    // pub fn scan_drivers(&mut self) {
    //     // 对于UI流畅性，最重要的是避免在UI线程上执行长时间运行的操作
    //     // 我们可以将扫描操作放到后台线程，但需要正确的线程安全实现
    //     // 现在我们暂时注释掉耗时操作，重点优化UI响应
    //     self.scan_in_progress = true;
    //     
    //     // 使用一个标记来表示后台正在进行扫描
    //     // 实际的扫描操作应该在另一个函数中使用适当的异步方法实现
    // }

    /// 加载SVG图标并转换为纹理
    fn load_svg_icon(&self, svg_path: &str, target_size: (u32, u32), ctx: &egui::Context) -> Option<egui::TextureHandle> {
        match std::fs::read(svg_path) {
            Ok(svg_data) => {
                // 解析SVG
                let opt = usvg::Options::default();
                match usvg::Tree::from_data(&svg_data, &opt) {
                    Ok(tree) => {
                        // 创建Pixmap进行渲染
                        let mut pixmap = Pixmap::new(target_size.0, target_size.1).unwrap();
                        
                        // 渲染SVG到Pixmap - 先填充透明背景
                        pixmap.fill(tiny_skia::Color::TRANSPARENT);
                        let rtree = resvg::Tree::from_usvg(&tree);
                        rtree.render(Transform::default(), &mut pixmap.as_mut());
                        
                        // 转换为egui的ColorImage
                        let pixels: Vec<u8> = pixmap
                            .pixels()
                            .iter()
                            .flat_map(|pixel| {
                                let rgba = pixel.demultiply();
                                [rgba.red(), rgba.green(), rgba.blue(), rgba.alpha()]
                            })
                            .collect();
                        
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [target_size.0 as usize, target_size.1 as usize],
                            &pixels,
                        );
                        
                        // 创建纹理
                        Some(ctx.load_texture(
                            "svg_icon",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ))
                    }
                    Err(e) => {
                        eprintln!("SVG解析失败 {}: {}", svg_path, e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("无法读取SVG文件 {}: {}", svg_path, e);
                None
            }
        }
    }
}

// 实现eframe::App trait
impl eframe::App for GuiApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // 使用默认颜色
        egui::Visuals::dark().panel_fill.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 恢复使用Windows DWM API实现阴影效果 - 仅在首次渲染时设置
        #[cfg(target_os = "windows")] {
            use winapi::um::dwmapi::{DwmSetWindowAttribute, DwmExtendFrameIntoClientArea};
            use winapi::um::uxtheme::MARGINS;
            use winapi::um::winuser::GetActiveWindow;

            // 定义DWM属性常量
            const DWMWA_NCRENDERING_POLICY: u32 = 2;
            const DWMNCRP_ENABLED: u32 = 2;

            // 尝试设置窗口属性以启用阴影
            use std::sync::atomic::{AtomicBool, Ordering};
            static SHADOW_INIT_DONE: AtomicBool = AtomicBool::new(false);

            if !SHADOW_INIT_DONE.load(Ordering::SeqCst) {
                // 使用活动窗口句柄作为回退方案
                let hwnd = unsafe { GetActiveWindow() };

                if !hwnd.is_null() {
                    unsafe {
                        // 启用非客户区渲染策略
                        let mut ncrp_enabled: u32 = DWMNCRP_ENABLED;
                        let result = DwmSetWindowAttribute(
                            hwnd,
                            DWMWA_NCRENDERING_POLICY,
                            &mut ncrp_enabled as *mut _ as *mut _,
                            std::mem::size_of::<u32>() as u32
                        );

                        if result == 0 { // S_OK
                            // 扩展边框以显示阴影 (使用负值来扩展阴影到窗口外部)
                            let margins = MARGINS {
                                cxLeftWidth: -1,
                                cxRightWidth: -1,
                                cyTopHeight: -1,
                                cyBottomHeight: -1,
                            };
                            let extend_result = DwmExtendFrameIntoClientArea(hwnd, &margins);

                            // 只有当扩展成功时才标记为已完成
                            if extend_result == 0 { // S_OK
                                SHADOW_INIT_DONE.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        }
        
        // 检查系统信息是否已异步加载完成
        if let Some(ref receiver) = self.system_info_rx {
            // 尝试接收结果，不阻塞UI线程
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(info) => {
                        println!("Received system info: {:?}", info);
                        println!("Monitor info: {:?}", info.monitor_info);
                        self.system_info = Some(info);
                        self.system_info_loading = false;
                    }
                    Err(e) => {
                        println!("System info error: {:?}", e);
                        self.system_info_error = Some(e);
                        self.system_info_loading = false;
                    }
                }
                // 移除receiver，因为我们已经收到了结果
                self.system_info_rx = None;
            }
        }
        
        // 请求定期重绘以确保UI响应
        ctx.request_repaint();
        
        
        // 侧边栏选项卡选择
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .min_width(200.0)
            .max_width(200.0)
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(0, 111, 201))
                .shadow(egui::epaint::Shadow::NONE)) // 重置阴影
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 应用标题
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        // 修正图标加载和显示 - 只加载一次
                        if self.title_icon.is_none() {
                            if let Ok(image_bytes) = std::fs::read("assets/icons/logo.png") {
                                if let Ok(image) = image::load_from_memory(&image_bytes) {
                                    let rgba = image.to_rgba8();
                                    let size = [image.width() as usize, image.height() as usize];
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        rgba.as_flat_samples().as_slice(),
                                    );
                                    let texture = ui.ctx().load_texture(
                                        "title_icon",
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    );
                                    self.title_icon = Some(texture);
                                }
                            }
                        }
                        

                        
                        // 显示图标
                        if let Some(ref texture) = self.title_icon {
                            ui.image((texture.id(), egui::Vec2::new(64.0, 64.0)));
                        }
                        ui.label(egui::RichText::new("仓鼠驱动管家").color(egui::Color32::WHITE).size(18.0)); // 应用标题
                    });
                    ui.add_space(4.0); // 在标题和功能面板之间添加间距
                    
                    // 为每个按钮创建自定义样式，设置选中时的背景色和文字颜色
                    let selected_bg_color = egui::Color32::WHITE; // 选中时背景色为白色
                    let selected_fg_color = egui::Color32::from_rgb(13, 160, 253); // 选中时文字颜色为侧边栏蓝色
                    let hover_bg_color = egui::Color32::from_rgba_premultiplied(245, 245, 245, 8); // 悬停时背景色为接近完全透明的白色 (rgba(245,245,245,0.03))
                    
                    ui.visuals_mut().selection.bg_fill = selected_bg_color;
                    ui.visuals_mut().selection.stroke.color = selected_fg_color; // 设置选中状态的前景色
                    
                    // 按钮菜单 - 使用按钮并设置宽度以填满侧边栏
                    ui.add_space(2.0); // 小间距
                    
                    // 获取当前UI上下文的可用宽度
                    let sidebar_width = ui.available_width();
                    
                    // 创建填满宽度的可选择按钮，支持悬停和点击效果
                    // 按钮文字居中，选中和悬停时背景色为白色，文字颜色保持egui框架默认色
                    
                    // 使用自定义字体大小
                    let font_id = egui::FontId::new(14.0, egui::FontFamily::Proportional);
                    // 概览按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let _is_selected = self.selected_tab == AppTab::Overview;
                        
                        // 绘制选中状态背景
                        if _is_selected {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                        }
                        // 绘制悬停状态背景
                        else if response.hovered() {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                        }
                        
                        ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "电脑概览",
                                font_id.clone(),
                                if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                            );
                        if response.clicked() {
                            self.selected_tab = AppTab::Overview;
                        }
                    });

                    // 驱动安装按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let _is_selected = self.selected_tab == AppTab::DriverInstall;
                        
                        // 绘制选中状态背景
                        if _is_selected {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                        }
                        // 绘制悬停状态背景
                        else if response.hovered() {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                        }
                        
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "驱动安装",
                            font_id.clone(),
                            if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::DriverInstall;
                        }
                    });

                    // 驱动管理按钮（可展开）
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let _is_selected = matches!(self.selected_tab, AppTab::DriverManagement | AppTab::DriverBackup | AppTab::DriverRestore | AppTab::DriverUninstall);
                        
                        // 绘制选中状态背景
                        if _is_selected {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                        }
                        // 绘制悬停状态背景
                        else if response.hovered() {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                        }
                        
                        // 绘制文本
                        let text = "驱动管理";
                        
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &text,
                            font_id.clone(),
                            if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                        );
                        
                        if response.clicked() {
                            self.driver_management_expanded = !self.driver_management_expanded;
                        }
                    });
                    
                    // 如果驱动管理菜单展开，显示子菜单
                    if self.driver_management_expanded {
                        // 备份驱动子菜单
                        ui.scope(|ui| {
                            let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 25.0), egui::Sense::click());
                            let _is_selected = self.selected_tab == AppTab::DriverBackup;
                            
                            // 绘制选中状态背景
                            if _is_selected {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                            }
                            // 绘制悬停状态背景
                            else if response.hovered() {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                            }
                            
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "💾备份驱动",
                                font_id.clone(),
                                if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                            );
                            
                            if response.clicked() {
                                self.selected_tab = AppTab::DriverBackup;
                            }
                        });
                        
                        // 恢复驱动子菜单
                        ui.scope(|ui| {
                            let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 25.0), egui::Sense::click());
                            let _is_selected = self.selected_tab == AppTab::DriverRestore;
                            
                            // 绘制选中状态背景
                            if _is_selected {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                            }
                            // 绘制悬停状态背景
                            else if response.hovered() {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                            }
                            
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🔄恢复驱动",
                                font_id.clone(),
                                if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                            );
                            
                            if response.clicked() {
                                self.selected_tab = AppTab::DriverRestore;
                            }
                        });
                        
                        // 卸载驱动子菜单
                        ui.scope(|ui| {
                            let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 25.0), egui::Sense::click());
                            let _is_selected = self.selected_tab == AppTab::DriverUninstall;
                            
                            // 绘制选中状态背景
                            if _is_selected {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                            }
                            // 绘制悬停状态背景
                            else if response.hovered() {
                                ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                            }
                            
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🗑️卸载驱动",
                                font_id.clone(),
                                if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                            );
                            
                            if response.clicked() {
                                self.selected_tab = AppTab::DriverUninstall;
                            }
                        });
                    }

                    // 系统、游戏运行组件按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let _is_selected = self.selected_tab == AppTab::SystemGameComponents;
                        
                        // 绘制选中状态背景
                        if _is_selected {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                        }
                        // 绘制悬停状态背景
                        else if response.hovered() {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                        }
                        
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "系统、游戏运行组件",
                            font_id.clone(),
                            if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::SystemGameComponents;
                        }
                    });



                    // 设置按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let _is_selected = self.selected_tab == AppTab::Settings;
                        
                        // 绘制选中状态背景
                        if _is_selected {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, selected_bg_color);
                        }
                        // 绘制悬停状态背景
                        else if response.hovered() {
                            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, hover_bg_color);
                        }
                        
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "设置",
                            font_id.clone(),
                            if _is_selected || response.hovered() { selected_fg_color } else { egui::Color32::from_rgb(242, 242, 242) }
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::Settings;
                        }
                    });
                    
                    // 在侧边栏底部添加版本号、GitHub按钮和网站链接
                    // 使用弹性空间将内容推到侧边栏底部
                    ui.add_space(ui.available_height() - 100.0); // 减去底部内容的高度
                    
                    // 加载GitHub图标 - 只加载一次
                    if self.github_icon.is_none() {
                        // 优先尝试加载SVG图标，如果失败则使用PNG图标作为备用
                        let svg_path = "assets/icons/GitHub.svg";
                        let png_paths = ["assets/icons/GitHub.png", "assets/icons/icon.png"];
                        
                        // 首先尝试加载SVG
                        if let Some(texture) = self.load_svg_icon(svg_path, (18, 18), ui.ctx()) {
                            self.github_icon = Some(texture);
                            println!("成功加载SVG GitHub图标: {}", svg_path);
                        } else {
                            // SVG加载失败，尝试PNG
                            for path in png_paths.iter() {
                                if let Ok(image_bytes) = std::fs::read(path) {
                                    if let Ok(image) = image::load_from_memory(&image_bytes) {
                                        let rgba = image.to_rgba8();
                                        let size = [image.width() as usize, image.height() as usize];
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                            size,
                                            rgba.as_flat_samples().as_slice(),
                                        );
                                        let texture = ui.ctx().load_texture(
                                            "github_icon",
                                            color_image,
                                            egui::TextureOptions::LINEAR,
                                        );
                                        self.github_icon = Some(texture);
                                        println!("成功加载PNG GitHub图标: {}", path);
                                        break;
                                    }
                                }
                            }
                        }
                        
                        // 如果所有图标都加载失败，记录错误
                        if self.github_icon.is_none() {
                            eprintln!("无法加载GitHub图标，请检查assets/icons/目录");
                        }
                    }
                    
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        
                        // 版本号和GitHub按钮水平排列 - 与侧边栏同宽并居中
                        ui.horizontal(|ui| {
                            // 添加弹性空间使内容居中
                            ui.add_space(ui.available_width() / 2.0 - 60.0); // 计算居中位置
                            
                            // 添加版本号
                            ui.label(egui::RichText::new("版本 v0.0.1")
                                .color(egui::Color32::BLACK) // 黑色文字
                                .size(14.0)); // 14px字体
                            
                            ui.add_space(5.0); // 版本号和GitHub按钮之间的间距
                            
                            // GitHub图标按钮
                            let github_response = ui.add_sized(
                                egui::Vec2::new(18.0, 18.0), // 按钮大小：18×18px
                                egui::Button::new("")
                                    .corner_radius(3.0)
                                    .stroke(egui::Stroke::NONE)
                                    .fill(egui::Color32::TRANSPARENT), // 透明背景色
                            );
                        
                            // 在按钮上显示GitHub图标
                            if let Some(ref texture) = self.github_icon {
                                let image_size = egui::Vec2::new(18.0, 18.0); // 图标大小：18×18px
                                let image_pos = github_response.rect.center() - image_size / 2.0;
                                let painter = ui.painter();
                                painter.image(
                                    texture.id(),
                                    egui::Rect::from_min_size(image_pos, image_size),
                                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                                    egui::Color32::BLACK
                                );
                            }
                            
                            if github_response.clicked() {
                                // 打开GitHub链接
                                if let Err(e) = webbrowser::open("https://github.com/Gautown/HamsterDrivers/releases/") {
                                    eprintln!("无法打开GitHub链接: {}", e);
                                }
                            }
                            if github_response.hovered() {
                                // 悬停时不需要额外绘制，正常状态的图标已经显示
                                // 这里可以添加其他悬停效果，如工具提示等
                            }
                        });
                    });
                    
                    // 网站链接水平居中
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() / 2.0 - 50.0); // 计算居中位置
                        
                        // 添加网站链接（可点击）
                        let website_response = ui.label(egui::RichText::new("www.GauTown.top")
                            .color(egui::Color32::from_rgb(0, 0, 0)) // 正常状态为黑色
                            .size(12.0)); // 较小的字体
                        
                        // 添加点击功能
                        if website_response.clicked() {
                            // 打开网站链接
                            if let Err(e) = webbrowser::open("https://gautown.top/") {
                                eprintln!("无法打开网站链接: {}", e);
                            }
                        }
                        
                        // 添加悬停效果 - 只改变文字颜色，不要背景色
                        if website_response.hovered() {
                            let painter = ui.painter_at(website_response.rect);
                            
                            // 悬停时显示黑色文字（覆盖在原有文字上）
                            painter.text(
                                website_response.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "www.GauTown.top",
                                egui::FontId::new(12.0, egui::FontFamily::Proportional),
                                egui::Color32::BLACK
                            );
                        }
                    });
                    
                    // 添加底部间距，使网站链接距离侧边栏底部16px
                    ui.add_space(16.0);
                });
            });

        // 顶部自定义标题栏（覆盖整窗体宽度）
        egui::TopBottomPanel::top("title_bar")
            .exact_height(36.0)
            .frame(egui::Frame::NONE) // 移除边框和阴影
            .show(ctx, |ui| {
                // 首先绘制整个标题栏背景
                let full_title_bar_rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    full_title_bar_rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_rgb(248, 248, 248),
                );
                
                ui.horizontal(|ui| {
                    let title_bar_rect = ui.available_rect_before_wrap();

                    // 拖拽区域为标题栏除去右侧按钮区域
                    // 两个按钮各32宽 + 内部间距6，留点额外空间
                    let button_area_width = 80.0;
                    let button_area_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(title_bar_rect.max.x - button_area_width, title_bar_rect.min.y),
                        egui::Vec2::new(button_area_width, title_bar_rect.height()),
                    );

                    let drag_area_rect = egui::Rect::from_min_max(
                        title_bar_rect.min,
                        egui::Pos2::new(button_area_rect.min.x, title_bar_rect.max.y),
                    );

                    let drag_response = ui.interact(drag_area_rect, egui::Id::new("title_bar_drag"), egui::Sense::click_and_drag());
                    if drag_response.drag_started() {
                        // 使用 eframe 提供的标准窗口拖动命令
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    // 按钮区域
                    #[allow(deprecated)] {
                        ui.allocate_ui_at_rect(button_area_rect, |ui| {
                            // 在按钮区域内部绘制背景
                            let button_area_inner_rect = ui.available_rect_before_wrap();
                            ui.painter().rect_filled(
                                button_area_inner_rect,
                                egui::CornerRadius::ZERO,
                                egui::Color32::from_rgb(248, 248, 248),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

                            // 关闭按钮
                            let close_response = ui.add_sized(
                                egui::Vec2::new(32.0, 36.0),
                                egui::Button::new("×")
                                    .corner_radius(3.0)
                                    .stroke(egui::Stroke::NONE)
                                    .fill(egui::Color32::from_rgb(248, 248, 248)),
                            );
                            if close_response.clicked() {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }

                            if close_response.hovered() {
                                let painter = ui.painter_at(close_response.rect);
                                painter.rect_filled(close_response.rect, egui::CornerRadius::same(3), egui::Color32::from_rgb(232, 17, 35));
                                painter.text(close_response.rect.center(), egui::Align2::CENTER_CENTER, "×", egui::FontId::proportional(14.0), egui::Color32::WHITE);
                            }

                            // 最小化按钮
                            let min_response = ui.add_sized(
                                egui::Vec2::new(32.0, 36.0),
                                egui::Button::new(egui::RichText::new("−").color(egui::Color32::from_rgb(33, 33, 33)))
                                    .corner_radius(3.0)
                                    .stroke(egui::Stroke::NONE)
                                    .fill(egui::Color32::from_rgb(248, 248, 248)),
                            );
                            if min_response.clicked() {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                            if min_response.hovered() {
                                let painter = ui.painter_at(min_response.rect);
                                painter.rect_filled(min_response.rect, egui::CornerRadius::same(3), egui::Color32::from_gray(200));
                                painter.text(min_response.rect.center(), egui::Align2::CENTER_CENTER, "−", egui::FontId::proportional(14.0), egui::Color32::from_rgb(33, 33, 33));
                            }

                            // ≡按钮（菜单按钮）
                            let menu_response = ui.add_sized(
                                egui::Vec2::new(32.0, 36.0),
                                egui::Button::new(egui::RichText::new("☰").size(16.0).color(egui::Color32::from_rgb(33, 33, 33))) // 设置16px字体和深灰色
                                    .corner_radius(3.0)
                                    .stroke(egui::Stroke::NONE)
                                    .fill(egui::Color32::from_rgb(248, 248, 248)),
                            );
                            if menu_response.clicked() {
                                // 这里可以添加菜单功能
                                println!("菜单按钮被点击");
                            }
                            if menu_response.hovered() {
                                let painter = ui.painter_at(menu_response.rect);
                                painter.rect_filled(menu_response.rect, egui::CornerRadius::same(3), egui::Color32::from_gray(200));
                                painter.text(menu_response.rect.center(), egui::Align2::CENTER_CENTER, "☰", egui::FontId::proportional(16.0), egui::Color32::from_rgb(33, 33, 33));
                            }

                            // 💬按钮（聊天/消息按钮）
                            let chat_response = ui.add_sized(
                                egui::Vec2::new(32.0, 36.0),
                                egui::Button::new(egui::RichText::new("💬").size(16.0).color(egui::Color32::from_rgb(33, 33, 33))) // 设置16px字体和深灰色
                                    .corner_radius(3.0)
                                    .stroke(egui::Stroke::NONE)
                                    .fill(egui::Color32::from_rgb(248, 248, 248)),
                            );
                            if chat_response.clicked() {
                                // 这里可以添加聊天/消息功能
                                println!("聊天按钮被点击");
                            }
                            if chat_response.hovered() {
                                let painter = ui.painter_at(chat_response.rect);
                                painter.rect_filled(chat_response.rect, egui::CornerRadius::same(3), egui::Color32::from_gray(200));
                                painter.text(chat_response.rect.center(), egui::Align2::CENTER_CENTER, "💬", egui::FontId::proportional(16.0), egui::Color32::from_rgb(33, 33, 33));
                            }
                            });
                        });
                    }
                });
            });

        // 主内容区域
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::WHITE).shadow(egui::epaint::Shadow::NONE))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 添加左边距16px，右边预留滚动条位置
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            // 为右边滚动条预留空间
                            let scrollbar_width = 20.0;
                            let available_width = ui.available_width();
                            let content_width = if available_width > scrollbar_width {
                                available_width - scrollbar_width
                            } else {
                                available_width
                            };
                            ui.set_width(content_width);
                            
                            // 内容继续在这里
                            match self.selected_tab {
                                AppTab::Overview => {
                                    ui.heading("系统概览");
                            
                                    // 检查是否需要加载系统信息（仅在首次显示概览页面时）
                                    if self.system_info.is_none() && !self.system_info_loading {
                                        self.system_info_loading = true;
                                        // 在后台线程中加载系统信息，避免UI卡顿
                                        let (tx, rx) = mpsc::channel();
                                        thread::spawn(move || {
                                            let result = SystemInfo::new();
                                            let _ = tx.send(result);
                                        });
                                        self.system_info_rx = Some(rx);
                                    }
                                
                                    if let Some(ref sys_info) = self.system_info {
                                        // 显示操作系统信息
                                        if let Some(ref os_name) = sys_info.os_name {
                                            ui.label(format!("操作系统: {}", os_name));
                                        }
                                        
                                        // 显示版本信息：使用更准确的方法获取Windows版本
                                        if let Some(ref os_name) = sys_info.os_name {
                                            // 根据操作系统名称和版本号判断Windows版本
                                            let version_display = if let Some(ref os_version) = sys_info.os_version {
                                                SystemInfo::get_windows_version_display(os_name, os_version)
                                            } else {
                                                "未知版本".to_string()
                                            };
                                            ui.label(format!("版本: {}", version_display));
                                        }
                                        
                                        // 显示操作系统版本号
                                        if let Some(ref os_version) = sys_info.os_version {
                                            ui.label(format!("版本号: {}", os_version));
                                        }
                                        
                                        // 显示内部版本号
                                        if let Some(ref os_version_formatted) = sys_info.os_version_formatted {
                                            ui.label(format!("内部版本: {}", os_version_formatted));
                                        }
                                        
                                        ui.separator();
                                        
                                        // 显示硬件信息
                                        if let Some(ref manufacturer) = sys_info.manufacturer {
                                            ui.label(format!("制造商: {}", manufacturer));
                                        }
                                        if let Some(ref motherboard) = sys_info.motherboard {
                                            ui.label(format!("主板: {}", motherboard));
                                        }
                                        if let Some(ref cpu) = sys_info.cpu {
                                            ui.label(format!("CPU: {}", cpu));
                                        }
                                        
                                        ui.separator();
                                        
                                        // 显示内存信息
                                        ui.label("内存信息:");
                                        for mem in &sys_info.memory_info {
                                            ui.label(format!("  {}", mem));
                                        }
                                        
                                        ui.separator();
                                        
                                        // 显示磁盘信息
                                        ui.label("磁盘信息:");
                                        for disk in &sys_info.disk_info {
                                            ui.label(format!("  {}", disk));
                                        }
                                        
                                        ui.separator();
                                        
                                        // 显示其他硬件信息
                                        ui.label("网络适配器:");
                                        for adapter in &sys_info.network_adapters {
                                            ui.label(format!("  {}", adapter));
                                        }
                                        
                                        ui.separator();
                                        
                                        ui.label("显卡信息:");
                                        for gpu in &sys_info.gpu_info {
                                            ui.label(format!("  {}", gpu));
                                        }
                                        
                                        ui.separator();
                                        
                                        ui.label("显示器信息:");
                                        for monitor in &sys_info.monitor_info {
                                            ui.label(format!("  {}", monitor));
                                        }
                                    } else if self.system_info_loading {
                                        ui.label("正在加载系统信息...");
                                    } else if let Some(ref error) = self.system_info_error {
                                        ui.label("加载系统信息失败:");
                                        ui.label(format!("  {}", error));
                                    } else {
                                        ui.label("点击切换到此页面以加载系统信息");
                                    }
                                },
                                _ => {
                                    show_advanced_features(ctx, self);
                                }
                            }
                        });
                    });
                });
            });

        // 只在需要时刷新UI，避免不必要的重绘
    }
}

// UI中的高级功能界面
fn show_advanced_features(ctx: &egui::Context, state: &mut GuiApp) {
    match state.selected_tab {
        AppTab::DriverInstall => show_driver_install_view(ctx, state),
        AppTab::DriverBackup => show_backup_driver_view(ctx, state),
        AppTab::DriverRestore => show_restore_driver_view(ctx, state),
        AppTab::DriverUninstall => show_uninstall_driver_view(ctx, state),
        AppTab::SystemGameComponents => show_system_game_components_view(ctx, state),
        AppTab::BackupRestore => show_backup_view(ctx, state),
        _ => {}
    }
}

// 系统、游戏运行组件视图
fn show_system_game_components_view(ctx: &egui::Context, _state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("系统、游戏运行组件");

        ui.label("这里将显示系统组件和游戏运行组件的管理功能");
        
        ui.separator();
        
        ui.label("系统组件：");
        if ui.button("检查系统组件状态").clicked() {
            // 这里可以触发系统组件检查
        }
        
        ui.separator();
        
        ui.label("游戏运行组件：");
        if ui.button("检查游戏运行库").clicked() {
            // 这里可以触发游戏运行库检查
        }
        
        if ui.button("安装缺失的组件").clicked() {
            // 这里可以触发组件安装
        }
    });
}

fn show_backup_view(ctx: &egui::Context, _state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("备份与恢复");

        if ui.button("备份选中驱动").clicked() {
            // 这里可以触发备份操作
        }

        if ui.button("恢复驱动").clicked() {
            // 这里可以触发恢复操作
        }

        ui.label("备份历史记录将在这里显示");
    });
}

// fn show_dependency_view(ctx: &egui::Context, state: &mut GuiApp) {
//     egui::CentralPanel::default().show(ctx, |ui| {
//         ui.heading("驱动依赖关系分析");

//         if ui.button("分析依赖关系").clicked() {
//             let _ = state.dependency_analyzer.analyze_dependencies(&state.drivers);
//         }

//         // 显示循环依赖
//         let circular = state.dependency_analyzer.find_circular_dependencies();
//         if !circular.is_empty() {
//             ui.colored_label(egui::Color32::RED, "⚠️ 发现循环依赖:");
//             for cycle in circular {
//                 let cycle_str: String = cycle.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" → ");
//                 ui.label(format!("➜ {}", cycle_str));
//             }
//         }

//         // 依赖关系图
//         if let Some(selected_idx) = state.selected_driver {
//             if let Some(driver) = state.drivers.get(selected_idx) {
//                 let chain = state.dependency_analyzer.get_dependency_chain(&driver.name);
//                 ui.collapsing("依赖链", |ui| {
//                     for (i, driver_name) in chain.iter().enumerate() {
//                         ui.horizontal(|ui| {
//                             ui.label(format!("{}. {}", i + 1, driver_name));
//                             if ui.small_button("查看").clicked() {
//                                 // 选择该驱动
//                             }
//                         });
//                     }
//                 });
//             }
//         }
//     });
// }



fn show_backup_driver_view(ctx: &egui::Context, state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("备份驱动");
        
        if state.drivers.is_empty() {
            ui.label("没有可用的驱动信息，请先扫描系统驱动");
            return;
        }
        
        // 显示驱动列表
        ui.label("选择要备份的驱动:");
        
        for (i, driver) in state.drivers.iter().enumerate() {
            ui.horizontal(|ui| {
                // 选择框
                let mut is_selected = state.selected_driver == Some(i);
                if ui.checkbox(&mut is_selected, "").changed() {
                    if is_selected {
                        state.selected_driver = Some(i);
                    } else {
                        state.selected_driver = None;
                    }
                }
                
                // 驱动信息
                ui.vertical(|ui| {
                    ui.label(format!("名称: {}", driver.name));
                    ui.label(format!("显示名称: {}", driver.display_name));
                    ui.label(format!("版本: {}", driver.version));
                    ui.label(format!("状态: {:?}", driver.status));
                });
                
                // 备份按钮
                if ui.button("备份").clicked() {
                    let result = state.driver_management.backup_driver(driver);
                    if result.success {
                        ui.colored_label(egui::Color32::GREEN, "✓ 备份成功");
                    } else {
                        ui.colored_label(egui::Color32::RED, format!("✗ 备份失败: {}", result.message));
                    }
                }
            });
            ui.separator();
        }
        
        // 批量备份按钮
        if ui.button("备份所有驱动").clicked() {
            for driver in &state.drivers {
                let result = state.driver_management.backup_driver(driver);
                if result.success {
                    ui.colored_label(egui::Color32::GREEN, format!("✓ {} 备份成功", driver.name));
                } else {
                    ui.colored_label(egui::Color32::RED, format!("✗ {} 备份失败: {}", driver.name, result.message));
                }
            }
        }
        
        ui.separator();
        
        // 备份历史记录
        ui.heading("备份历史记录");
        let history = state.driver_management.get_backup_history();
        if !history.is_empty() {
            for record in history.iter() {
                ui.horizontal(|ui| {
                    if record.success {
                        ui.colored_label(egui::Color32::GREEN, "✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗");
                    }
                    ui.vertical(|ui| {
                        ui.label(format!("驱动: {}", record.driver_name));
                        ui.label(format!("时间: {}", record.timestamp));
                        ui.label(format!("结果: {}", record.message));
                    });
                });
                ui.separator();
            }
            
            if ui.button("清空备份历史").clicked() {
                state.driver_management.clear_backup_history();
            }
        } else {
            ui.label("暂无备份记录");
        }
    });
}

fn show_restore_driver_view(ctx: &egui::Context, state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("恢复驱动");
        
        // 获取备份文件列表
        let backup_files = state.driver_management.get_backup_files();
        
        if backup_files.is_empty() {
            ui.label("没有可用的备份文件");
            return;
        }
    
        ui.label("选择备份文件进行恢复:");
        
        for (i, backup_file) in backup_files.iter().enumerate() {
            ui.horizontal(|ui| {
                // 选择框
                let mut is_selected = state.selected_backup_file == Some(i);
                if ui.checkbox(&mut is_selected, "").changed() {
                    if is_selected {
                        state.selected_backup_file = Some(i);
                    } else {
                        state.selected_backup_file = None;
                    }
                }
                
                // 备份文件信息
                ui.vertical(|ui| {
                    if let Some(file_name) = backup_file.file_name().and_then(|s| s.to_str()) {
                        ui.label(format!("文件: {}", file_name));
                    }
                    if let Ok(metadata) = std::fs::metadata(backup_file) {
                        if let Ok(modified) = metadata.modified() {
                            let modified_time = chrono::DateTime::<chrono::Local>::from(modified);
                            ui.label(format!("修改时间: {}", modified_time.format("%Y-%m-%d %H:%M:%S")));
                        }
                    }
                });
                
                // 恢复按钮
                if ui.button("恢复").clicked() {
                    let result = state.driver_management.restore_driver(backup_file);
                    if result.success {
                        ui.colored_label(egui::Color32::GREEN, "✓ 恢复成功");
                    } else {
                        ui.colored_label(egui::Color32::RED, format!("✗ 恢复失败: {}", result.message));
                    }
                }
            });
            ui.separator();
        }
        
        ui.separator();
        
        // 恢复历史记录
        ui.heading("恢复历史记录");
        let history = state.driver_management.get_restore_history();
        if !history.is_empty() {
            for record in history.iter() {
                ui.horizontal(|ui| {
                    if record.success {
                        ui.colored_label(egui::Color32::GREEN, "✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗");
                    }
                    ui.vertical(|ui| {
                        ui.label(format!("驱动: {}", record.driver_name));
                        ui.label(format!("时间: {}", record.timestamp));
                        ui.label(format!("结果: {}", record.message));
                    });
                });
                ui.separator();
            }
            
            if ui.button("清空恢复历史").clicked() {
                state.driver_management.clear_restore_history();
            }
        } else {
            ui.label("暂无恢复记录");
        }
    });
}

fn show_uninstall_driver_view(ctx: &egui::Context, state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("卸载驱动");
        
        if state.drivers.is_empty() {
            ui.label("没有可用的驱动信息，请先扫描系统驱动");
            return;
        }
        
        ui.label("选择要卸载的驱动:");
        
        for (i, driver) in state.drivers.iter().enumerate() {
            ui.horizontal(|ui| {
                // 选择框
                let mut is_selected = state.selected_driver == Some(i);
                if ui.checkbox(&mut is_selected, "").changed() {
                    if is_selected {
                        state.selected_driver = Some(i);
                    } else {
                        state.selected_driver = None;
                    }
                }
                
                // 驱动信息
                ui.vertical(|ui| {
                    ui.label(format!("名称: {}", driver.name));
                    ui.label(format!("显示名称: {}", driver.display_name));
                    ui.label(format!("版本: {}", driver.version));
                    ui.label(format!("状态: {:?}", driver.status));
                });
                
                // 卸载按钮
                if ui.button("卸载").clicked() {
                    let result = state.driver_management.uninstall_driver(driver);
                    if result.success {
                        ui.colored_label(egui::Color32::GREEN, "✓ 卸载成功");
                    } else {
                        ui.colored_label(egui::Color32::RED, format!("✗ 卸载失败: {}", result.message));
                    }
                }
            });
            ui.separator();
        }
        
        ui.separator();
        
        // 卸载历史记录
        ui.heading("卸载历史记录");
        let history = state.driver_management.get_uninstall_history();
        if !history.is_empty() {
            for record in history.iter() {
                ui.horizontal(|ui| {
                    if record.success {
                        ui.colored_label(egui::Color32::GREEN, "✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗");
                    }
                    ui.vertical(|ui| {
                        ui.label(format!("驱动: {}", record.driver_name));
                        ui.label(format!("时间: {}", record.timestamp));
                        ui.label(format!("结果: {}", record.message));
                    });
                });
                ui.separator();
            }
            
            if ui.button("清空卸载历史").clicked() {
                state.driver_management.clear_uninstall_history();
            }
        } else {
            ui.label("暂无卸载记录");
        }
    });
}

fn show_driver_install_view(ctx: &egui::Context, state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("驱动安装");
        
        // 扫描目录设置
        ui.horizontal(|ui| {
            ui.label("扫描目录:");
            ui.text_edit_singleline(&mut state.scan_directory);
            if ui.button("浏览").clicked() {
                // 这里可以添加目录选择对话框
            }
            
            if ui.button("扫描驱动文件").clicked() {
                let path = std::path::PathBuf::from(&state.scan_directory);
                match state.driver_installer.scan_drivers_in_directory(&path) {
                    Ok(drivers) => {
                        state.scanned_drivers = drivers;
                    }
                    Err(e) => {
                        ui.colored_label(egui::Color32::RED, format!("扫描失败: {}", e));
                    }
                }
            }
        });
        
        ui.separator();
        
        // 显示扫描到的驱动文件
        if !state.scanned_drivers.is_empty() {
            ui.heading("可安装的驱动文件");
            
            for (i, driver) in state.scanned_drivers.iter().enumerate() {
                ui.horizontal(|ui| {
                    // 选择框
                    let mut is_selected = state.selected_install_driver == Some(i);
                    if ui.checkbox(&mut is_selected, "").changed() {
                        if is_selected {
                            state.selected_install_driver = Some(i);
                        } else {
                            state.selected_install_driver = None;
                        }
                    }
                    
                    // 驱动信息
                    ui.vertical(|ui| {
                        ui.label(format!("名称: {}", driver.display_name));
                        ui.label(format!("版本: {}", driver.version));
                        ui.label(format!("制造商: {}", driver.manufacturer));
                        ui.label(format!("安装方法: {:?}", driver.install_method));
                        ui.label(format!("签名状态: {}", driver.signature_status));
                    });
                    
                    // 安装按钮
                    if ui.button("安装").clicked() {
                        let result = state.driver_installer.install_driver(driver);
                        if result.success {
                            ui.colored_label(egui::Color32::GREEN, "✓ 安装成功");
                        } else {
                            ui.colored_label(egui::Color32::RED, format!("✗ 安装失败: {}", result.message));
                        }
                    }
                });
                ui.separator();
            }
            
            // 批量安装按钮
            if ui.button("批量安装所有驱动").clicked() {
                for driver in &state.scanned_drivers {
                    let result = state.driver_installer.install_driver(driver);
                    if result.success {
                        ui.colored_label(egui::Color32::GREEN, format!("✓ {} 安装成功", driver.name));
                    } else {
                        ui.colored_label(egui::Color32::RED, format!("✗ {} 安装失败: {}", driver.name, result.message));
                    }
                }
            }
        } else {
            ui.label("请先扫描目录中的驱动文件");
        }
        
        ui.separator();
        
        // 安装历史记录
        ui.heading("安装历史记录");
        let history = state.driver_installer.get_installation_history();
        if !history.is_empty() {
            for record in history.iter() {
                ui.horizontal(|ui| {
                    if record.success {
                        ui.colored_label(egui::Color32::GREEN, "✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗");
                    }
                    ui.vertical(|ui| {
                        ui.label(format!("驱动: {}", record.driver_name));
                        ui.label(format!("时间: {}", record.timestamp));
                        ui.label(format!("结果: {}", record.message));
                    });
                });
                ui.separator();
            }
            
            if ui.button("清空历史记录").clicked() {
                state.driver_installer.clear_installation_history();
            }
        } else {
            ui.label("暂无安装记录");
        }
    });
}

