// 将原来的ui.rs内容整合进来
use eframe::egui;
use crate::core::sysinfo::SystemInfo;
// 新增导入 image crate
use image::GenericImageView;
use std::sync::mpsc;
use std::thread;
#[allow(dead_code)]
pub struct GuiApp {
    // 使用core模块中的类型
    pub driver_service: crate::core::windows_api::driver_service::DriverService,
    pub dependency_analyzer: crate::core::features::dependency_analyzer::DependencyAnalyzer,
    pub signature_validator: crate::core::features::signature_validator::SignatureValidator,
    pub backup_manager: crate::core::features::backup_manager::BackupManager,
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
}

#[derive(PartialEq)]
enum AppTab {
    Overview,
    DriverList,
    Dependencies,
    Signatures,
    BackupRestore,
    Settings,
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
            // window drag handled natively on Windows
        })
    }

    pub fn scan_drivers(&mut self) {
        // 对于UI流畅性，最重要的是避免在UI线程上执行长时间运行的操作
        // 我们可以将扫描操作放到后台线程，但需要正确的线程安全实现
        // 现在我们暂时注释掉耗时操作，重点优化UI响应
        self.scan_in_progress = true;
        
        // 使用一个标记来表示后台正在进行扫描
        // 实际的扫描操作应该在另一个函数中使用适当的异步方法实现
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
                .fill(egui::Color32::from_rgb(233, 233, 233))
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
                        ui.label("仓鼠驱动管家"); // 应用标题
                    });
                    ui.add_space(4.0); // 在标题和功能面板之间添加间距
                    
                    // 为每个按钮创建自定义样式，设置选中时的背景色和文字颜色
                    let selected_bg_color = egui::Color32::from_rgb(248, 248, 248);
                    let selected_fg_color = egui::Color32::from_rgb(3, 3, 3); // 选中时文字颜色
                    
                    ui.visuals_mut().selection.bg_fill = selected_bg_color;
                    ui.visuals_mut().selection.stroke.color = selected_fg_color; // 设置选中状态的前景色
                    
                    // 按钮菜单 - 使用按钮并设置宽度以填满侧边栏
                    ui.add_space(2.0); // 小间距
                    
                    // 获取当前UI上下文的可用宽度
                    let sidebar_width = ui.available_width();
                    
                    // 创建填满宽度的可选择按钮，支持悬停和点击效果
                    // 按钮文字居中，选中和悬停时背景色为白色，文字颜色保持egui框架默认色
                    
                    // 修正 TextStyle 用法
                    let text_style = egui::TextStyle::Body;
                    // 概览按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let is_selected = self.selected_tab == AppTab::Overview;
                        let bg_color = if is_selected || response.hovered() {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(233, 233, 233)
                        };
                        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "概览",
                            ui.ctx().style().text_styles.get(&text_style).unwrap().clone(),
                            ui.style().visuals.text_color()
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::Overview;
                        }
                    });

                    // 驱动列表按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let is_selected = self.selected_tab == AppTab::DriverList;
                        let bg_color = if is_selected || response.hovered() {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(233, 233, 233)
                        };
                        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "驱动列表",
                            ui.ctx().style().text_styles.get(&text_style).unwrap().clone(),
                            ui.style().visuals.text_color()
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::DriverList;
                        }
                    });

                    // 依赖分析按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let is_selected = self.selected_tab == AppTab::Dependencies;
                        let bg_color = if is_selected || response.hovered() {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(233, 233, 233)
                        };
                        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "依赖分析",
                            ui.ctx().style().text_styles.get(&text_style).unwrap().clone(),
                            ui.style().visuals.text_color()
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::Dependencies;
                        }
                    });

                    // 签名验证按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let is_selected = self.selected_tab == AppTab::Signatures;
                        let bg_color = if is_selected || response.hovered() {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(233, 233, 233)
                        };
                        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "签名验证",
                            ui.ctx().style().text_styles.get(&text_style).unwrap().clone(),
                            ui.style().visuals.text_color()
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::Signatures;
                        }
                    });

                    // 备份恢复按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let is_selected = self.selected_tab == AppTab::BackupRestore;
                        let bg_color = if is_selected || response.hovered() {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(233, 233, 233)
                        };
                        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "备份恢复",
                            ui.ctx().style().text_styles.get(&text_style).unwrap().clone(),
                            ui.style().visuals.text_color()
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::BackupRestore;
                        }
                    });

                    // 设置按钮
                    ui.scope(|ui| {
                        let (rect, response) = ui.allocate_exact_size(egui::Vec2::new(sidebar_width, 30.0), egui::Sense::click());
                        let is_selected = self.selected_tab == AppTab::Settings;
                        let bg_color = if is_selected || response.hovered() {
                            egui::Color32::from_rgb(255, 255, 255)
                        } else {
                            egui::Color32::from_rgb(233, 233, 233)
                        };
                        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg_color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "设置",
                            ui.ctx().style().text_styles.get(&text_style).unwrap().clone(),
                            ui.style().visuals.text_color()
                        );
                        if response.clicked() {
                            self.selected_tab = AppTab::Settings;
                        }
                    });
                });
            });

        // 顶部自定义标题栏（覆盖整窗体宽度）
        egui::TopBottomPanel::top("title_bar")
            .exact_height(36.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let title_bar_rect = ui.available_rect_before_wrap();
                    // 绘制标题栏背景
                    ui.painter().rect_filled(
                        title_bar_rect,
                        egui::CornerRadius::ZERO,
                        egui::Color32::from_rgb(248, 248, 248),
                    );

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
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

                            // 关闭按钮
                            let close_response = ui.add_sized(
                                egui::Vec2::new(32.0, 30.0),
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
                                egui::Vec2::new(32.0, 30.0),
                                egui::Button::new("−")
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
                                        
                                        // 显示版本信息：版本：yyHx格式
                                        if let Some(ref os_version) = sys_info.os_version {
                                            // 解析版本号，例如 "10.0.19045"
                                            let version_parts: Vec<&str> = os_version.split('.').collect();
                                            if version_parts.len() >= 2 {
                                                let major_version = version_parts[0];
                                                let feature_version = version_parts[1];
                                                
                                                // 将Windows版本号转换为yyHx格式
                                                let yyhx_version = match (major_version, feature_version) {
                                                    ("10", "0") => "21H2".to_string(),  // Windows 10 21H2
                                                    ("10", "1") => "21H1".to_string(),  // Windows 10 21H1
                                                    ("10", "2") => "22H2".to_string(),  // Windows 11 22H2
                                                    ("10", "3") => "23H2".to_string(),  // Windows 11 23H2
                                                    ("10", "4") => "24H2".to_string(),  // Windows 11 24H2
                                                    ("10", "5") => "25H2".to_string(),  // Windows 11 25H2
                                                    ("6", "3") => "13H2".to_string(),   // Windows 8.1
                                                    ("6", "2") => "12H2".to_string(),   // Windows 8
                                                    ("6", "1") => "11H1".to_string(),   // Windows 7 SP1
                                                    ("6", "0") => "08H2".to_string(),   // Windows Vista SP2
                                                    ("5", "1") => "01H2".to_string(),   // Windows XP SP3
                                                    ("5", "0") => "00H1".to_string(),   // Windows 2000
                                                    _ => format!("{}H{}", major_version, feature_version)
                                                };
                                                
                                                ui.label(format!("版本: {}", yyhx_version));
                                            } else {
                                                ui.label(format!("版本: {}", os_version));
                                            }
                                        }
                                        
                                        // 显示版本号：操作系统版本
                                        if let Some(ref os_version) = sys_info.os_version {
                                            ui.label(format!("版本号: {}", os_version));
                                        }
                                        
                                        // 显示版本号：内部版本号
                                        if let Some(ref os_version_formatted) = sys_info.os_version_formatted {
                                            // 从格式化版本中提取内部版本号
                                            if let Some(build_start) = os_version_formatted.find("Build") {
                                                let build_part = &os_version_formatted[build_start..];
                                                ui.label(format!("版本号: {}", build_part));
                                            } else {
                                                ui.label(format!("版本号: {}", os_version_formatted));
                                            }
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
        AppTab::Dependencies => show_dependency_view(ctx, state),
        AppTab::Signatures => show_signature_view(ctx, state),
        AppTab::BackupRestore => show_backup_view(ctx, state),
        _ => {}
    }
}

// 添加缺失的函数
fn show_signature_view(ctx: &egui::Context, _state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("驱动签名验证");

        if ui.button("验证选中驱动的签名").clicked() {
            // 这里可以触发签名验证
        }

        ui.label("签名验证结果将在这里显示");
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

fn show_dependency_view(ctx: &egui::Context, state: &mut GuiApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("驱动依赖关系分析");

        if ui.button("分析依赖关系").clicked() {
            let _ = state.dependency_analyzer.analyze_dependencies(&state.drivers);
        }

        // 显示循环依赖
        let circular = state.dependency_analyzer.find_circular_dependencies();
        if !circular.is_empty() {
            ui.colored_label(egui::Color32::RED, "⚠️ 发现循环依赖:");
            for cycle in circular {
                let cycle_str: String = cycle.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" → ");
                ui.label(format!("➜ {}", cycle_str));
            }
        }

        // 依赖关系图
        if let Some(selected_idx) = state.selected_driver {
            if let Some(driver) = state.drivers.get(selected_idx) {
                let chain = state.dependency_analyzer.get_dependency_chain(&driver.name);
                ui.collapsing("依赖链", |ui| {
                    for (i, driver_name) in chain.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", i + 1, driver_name));
                            if ui.small_button("查看").clicked() {
                                // 选择该驱动
                            }
                        });
                    }
                });
            }
        }
    });
}