// src/driver_manager.rs
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Local};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriverStatus {
    Running,
    Stopped,
    Paused,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriverType {
    KernelMode,
    UserMode,
    FileSystem,
    Network,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub status: DriverStatus,
    pub driver_type: DriverType,
    pub start_type: String,
    pub binary_path: String,
    pub version: String,
    pub company: String,
    pub signed: bool,
    pub signature_status: String,
    pub last_updated: DateTime<Local>,
    pub dependencies: Vec<String>,
    pub load_order: u32,
}

#[derive(Debug)]
pub struct DriverManager {
    drivers: Vec<DriverInfo>,
    filtered_drivers: Vec<DriverInfo>,
    selected_driver: Option<usize>,
    filter_text: String,
    filter_type: Option<DriverType>,
    filter_status: Option<DriverStatus>,
    show_only_signed: bool,
    show_only_unsigned: bool,
}

impl DriverManager {
    pub fn new() -> Self {
        Self {
            drivers: Vec::new(),
            filtered_drivers: Vec::new(),
            selected_driver: None,
            filter_text: String::new(),
            filter_type: None,
            filter_status: None,
            show_only_signed: false,
            show_only_unsigned: false,
        }
    }
    
    // 枚举系统驱动程序（Windows API实现）
    pub fn refresh_drivers(&mut self) -> Result<(), String> {
        self.drivers.clear();
        
        // Windows服务管理器枚举驱动服务
        // 这里需要调用Windows API，实际实现会使用windows crate
        // 简化示例：
        self.drivers = vec![
            DriverInfo {
                name: "1394ohci".to_string(),
                display_name: "1394 OHCI Compliant Host Controller".to_string(),
                description: "提供对1394总线的支持".to_string(),
                status: DriverStatus::Running,
                driver_type: DriverType::KernelMode,
                start_type: "手动".to_string(),
                binary_path: "System32\\drivers\\1394ohci.sys".to_string(),
                version: "10.0.19041.1".to_string(),
                company: "Microsoft Corporation".to_string(),
                signed: true,
                signature_status: "已验证".to_string(),
                last_updated: Local::now(),
                dependencies: vec![],
                load_order: 0,
            },
            // 更多驱动...
        ];
        
        self.apply_filters();
        Ok(())
    }
    
    fn apply_filters(&mut self) {
        self.filtered_drivers = self.drivers
            .iter()
            .filter(|driver| {
                let text_match = self.filter_text.is_empty() ||
                    driver.name.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                    driver.display_name.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                    driver.description.to_lowercase().contains(&self.filter_text.to_lowercase());
                
                let type_match = self.filter_type.as_ref()
                    .map_or(true, |filter_type| &driver.driver_type == filter_type);
                
                let status_match = self.filter_status.as_ref()
                    .map_or(true, |filter_status| &driver.status == filter_status);
                
                let signed_match = if self.show_only_signed {
                    driver.signed
                } else if self.show_only_unsigned {
                    !driver.signed
                } else {
                    true
                };
                
                text_match && type_match && status_match && signed_match
            })
            .cloned()
            .collect();
    }
}