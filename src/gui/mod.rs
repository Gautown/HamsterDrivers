// 将原来的ui.rs内容整合进来
use eframe::egui;
use crate::Core::sysinfo::SystemInfo;
pub struct GuiApp {
    // 使用Core模块中的类型
    pub driver_service: crate::Core::DriverService,
    pub dependency_analyzer: crate::Core::DependencyAnalyzer,
    pub signature_validator: crate::Core::SignatureValidator,
    pub backup_manager: crate::Core::BackupManager,
    pub selected_tab: AppTab,
    pub drivers: Vec<crate::Core::driver_manager::DriverInfo>,
    pub backup_history: Vec<String>,
    pub scan_in_progress: bool,
    pub selected_driver: Option<usize>,
    pub system_info: Option<SystemInfo>,
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
        let system_info = match SystemInfo::new() {
            Ok(info) => Some(info),
            Err(e) => {
                eprintln!("Failed to get system info: {}", e);
                None
            }
        };
        
        Ok(Self {
            driver_service: crate::Core::DriverService::new()?,
            dependency_analyzer: crate::Core::DependencyAnalyzer::new(),
            signature_validator: crate::Core::SignatureValidator::new(),
            backup_manager: crate::Core::BackupManager::new()?,
            selected_tab: AppTab::Overview,
            drivers: Vec::new(),
            backup_history: Vec::new(),
            scan_in_progress: false,
            selected_driver: None,
            system_info,
        })
    }

    pub fn scan_drivers(&mut self) {
        self.scan_in_progress = true;

        match self.driver_service.enumerate_drivers() {
            Ok(driver_services) => {
                self.drivers = driver_services.into_iter().map(|ds| crate::Core::DriverInfo {
                    name: ds.name,
                    display_name: ds.display_name,
                    description: "Mock Description".to_string(),
                    status: ds.status,
                    driver_type: crate::Core::DriverType::KernelMode,
                    start_type: ds.start_type,
                    binary_path: ds.binary_path,
                    version: "1.0.0.0".to_string(),
                    company: "Mock Company".to_string(),
                    signed: true,
                    signature_status: "Valid".to_string(),
                    last_updated: chrono::Local::now(),
                    dependencies: vec![],
                    load_order: 1,
                }).collect();

                let _ = self.dependency_analyzer.analyze_dependencies(&self.drivers);
            }
            Err(e) => eprintln!("Failed to scan drivers: {}", e),
        }

        self.scan_in_progress = false;
    }
}

// 实现eframe::App trait
impl eframe::App for GuiApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // 使用默认颜色
        egui::Visuals::dark().panel_fill.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 启用Windows阴影效果
        #[cfg(target_os = "windows")] {
            use winapi::um::winuser::FindWindowA;
            use winapi::um::dwmapi::{DwmSetWindowAttribute, DwmExtendFrameIntoClientArea, DwmEnableBlurBehindWindow};
            use winapi::um::uxtheme::MARGINS;
            use winapi::shared::minwindef::TRUE;
            use std::ffi::CString;
            
            // 定义DWM属性常量
            const DWMWA_NCRENDERING_POLICY: u32 = 2;
            const DWMWA_TRANSITIONS_FORCEDISABLED: u32 = 3;
            const DWMNCRP_ENABLED: u32 = 2;
            
            // 仅在第一次更新时启用阴影
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                // 通过窗口类名和标题查找窗口句柄
                // 尝试多种方法获取窗口句柄
                let mut hwnd = std::ptr::null_mut();
                
                // 方法1: 尝试通过标题查找（使用应用名称）
                let title_name = CString::new("Hamster Drivers Manager").unwrap();
                hwnd = unsafe { FindWindowA(std::ptr::null(), title_name.as_ptr()) };
                
                // 方法2: 如果按标题找不到，则尝试使用GetActiveWindow
                if hwnd.is_null() {
                    use winapi::um::winuser::GetActiveWindow;
                    hwnd = unsafe { GetActiveWindow() };
                }
                
                if !hwnd.is_null() {
                    unsafe {
                        // 临时禁用过渡动画以避免视觉闪烁
                        let mut disabled = TRUE;
                        let _ = DwmSetWindowAttribute(
                            hwnd,
                            DWMWA_TRANSITIONS_FORCEDISABLED,
                            &mut disabled as *mut _ as *mut _,
                            std::mem::size_of::<u32>() as u32
                        );
                        
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
                            let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
                        }
                    }
                }
            });
        }
        
        
        // 侧边栏选项卡选择
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .min_width(200.0)
            .max_width(200.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(233, 233, 233)))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 应用标题
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        // 添加2px的上边距
                        ui.add_space(8.0);
                        // 尝试加载图标文件作为标题图标
                        if let Ok(image_bytes) = std::fs::read("assets/icons/logo.png") {
                            if let Ok(image) = image::load_from_memory(&image_bytes) {
                                let texture = ui.ctx().load_texture(
                                    "title_icon", 
                                    egui::ColorImage::from_rgba_unmultiplied(
                                        [image.width() as usize, image.height() as usize],
                                        image.to_rgba8().as_flat_samples().as_slice(),
                                    ),
                                    egui::TextureOptions::LINEAR,
                                );
                                let sized_texture = egui::load::SizedTexture::new(texture.id(), egui::Vec2::new(48.0, 48.0));
                        ui.image(sized_texture); // 调整图标大小
                            }
                        }
                        ui.label("仓鼠驱动管家"); // 应用标题
                    });
                    ui.add_space(8.0); // 在标题和功能面板之间添加间距
                    ui.label("功能面板");
                    
                    // 为每个按钮创建自定义样式，设置选中时的背景色和文字颜色
                    let selected_bg_color = egui::Color32::from_rgb(248, 248, 248);
                    let selected_fg_color = egui::Color32::from_rgb(3, 3, 3); // 选中时文字颜色
                    
                    ui.visuals_mut().selection.bg_fill = selected_bg_color;
                    ui.visuals_mut().selection.stroke.color = selected_fg_color; // 设置选中状态的前景色
                    
                    // 按钮菜单
                    ui.selectable_value(&mut self.selected_tab, AppTab::Overview, "概览");
                    ui.selectable_value(&mut self.selected_tab, AppTab::DriverList, "驱动列表");
                    ui.selectable_value(&mut self.selected_tab, AppTab::Dependencies, "依赖分析");
                    ui.selectable_value(&mut self.selected_tab, AppTab::Signatures, "签名验证");
                    ui.selectable_value(&mut self.selected_tab, AppTab::BackupRestore, "备份恢复");
                    ui.selectable_value(&mut self.selected_tab, AppTab::Settings, "设置");
                });
            });

        // 主内容区域
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::WHITE))
            .show(ctx, |ui| {
            ui.vertical(|ui| {
                // 顶部标题栏
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(248, 248, 248)) // 浅灰色标题栏
                    .inner_margin(egui::Margin::symmetric(10, 0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // 拖拽区域 - 点击并拖拽移动窗口
                            let title_response = ui.allocate_response(
                                egui::Vec2::new(ui.available_width()-70.0, 30.0), // 为按钮预留空间
                                egui::Sense::click_and_drag(),
                            );
                            
                            if title_response.drag_started() {
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::new(2.0, 0.0); // 设置按钮间距
                                
                                // 关闭按钮
                                if ui.add_sized(egui::Vec2::new(32.0, 30.0), egui::Button::new("×").rounding(3.0).fill(
                                    if ui.visuals().dark_mode {
                                        egui::Color32::from_rgb(200, 60, 60) // 暗模式下的深红
                                    } else {
                                        egui::Color32::from_rgb(248, 248, 248) // 浅灰色
                                    }
                                )).clicked() {
                                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                
                                // 最小化按钮
                                if ui.add_sized(egui::Vec2::new(32.0, 30.0), egui::Button::new("−").rounding(3.0).fill(
                                    if ui.visuals().dark_mode {
                                        egui::Color32::from_gray(80) // 暗模式下的深灰
                                    } else {
                                        egui::Color32::from_rgb(248, 248, 248) // 浅灰色
                                    }
                                )).clicked() {
                                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                                }
                            });
                        });
                    });
                
                match self.selected_tab {
                    AppTab::Overview => {
                        ui.heading("系统概览");
                        
                        if let Some(ref sys_info) = self.system_info {
                            // 显示操作系统信息
                            if let Some(ref os_name) = sys_info.os_name {
                                ui.label(format!("操作系统: {}", os_name));
                            }
                            if let Some(ref os_version) = sys_info.os_version {
                                ui.label(format!("系统版本: {}", os_version));
                            }
                            if let Some(ref os_version_formatted) = sys_info.os_version_formatted {
                                ui.label(format!("版本标识: {}", os_version_formatted));
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
                        } else {
                            ui.label("正在加载系统信息...");
                        }
                    },
                    _ => {
                        show_advanced_features(ctx, self);
                    }
                };
            });
        });

        // 刷新UI
        ctx.request_repaint();
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
                let cycle_str: String = cycle.join(" → ");
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