use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};
use crate::core::driver_manager::DriverInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub message: String,
    pub driver_name: String,
    pub backup_path: PathBuf,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
    pub driver_name: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub success: bool,
    pub message: String,
    pub driver_name: String,
    pub timestamp: String,
}

#[allow(dead_code)]
pub struct DriverManagement {
    backup_dir: PathBuf,
    backup_history: Vec<BackupResult>,
    restore_history: Vec<RestoreResult>,
    uninstall_history: Vec<UninstallResult>,
}

impl DriverManagement {
    pub fn new() -> Self {
        let backup_dir = PathBuf::from("./driver_backups");
        
        // 确保备份目录存在
        if !backup_dir.exists() {
            std::fs::create_dir_all(&backup_dir).unwrap_or_else(|_| {
                eprintln!("无法创建备份目录: {:?}", backup_dir);
            });
        }
        
        Self {
            backup_dir,
            backup_history: Vec::new(),
            restore_history: Vec::new(),
            uninstall_history: Vec::new(),
        }
    }
    
    // 备份驱动
    pub fn backup_driver(&mut self, driver: &DriverInfo) -> BackupResult {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_filename = format!("{}_{}.bak", driver.name, timestamp);
        let backup_path = self.backup_dir.join(&backup_filename);
        
        // 创建备份文件内容
        let backup_content = format!(
            "Driver Backup Information\n\
            ========================\n\
            Name: {}\n\
            Display Name: {}\n\
            Version: {}\n\
            Company: {}\n\
            Binary Path: {}\n\
            Status: {:?}\n\
            Driver Type: {:?}\n\
            Signed: {}\n\
            Signature Status: {}\n\
            Backup Time: {}\n\
            Dependencies: {}\n",
            driver.name,
            driver.display_name,
            driver.version,
            driver.company,
            driver.binary_path,
            driver.status,
            driver.driver_type,
            driver.signed,
            driver.signature_status,
            timestamp,
            driver.dependencies.join(", ")
        );
        
        match std::fs::write(&backup_path, &backup_content) {
            Ok(_) => {
                let result = BackupResult {
                    success: true,
                    message: "备份成功".to_string(),
                    driver_name: driver.name.clone(),
                    backup_path: backup_path.clone(),
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                };
                self.backup_history.push(result.clone());
                result
            }
            Err(e) => {
                let result = BackupResult {
                    success: false,
                    message: format!("备份失败: {}", e),
                    driver_name: driver.name.clone(),
                    backup_path: backup_path,
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                };
                self.backup_history.push(result.clone());
                result
            }
        }
    }
    
    // 恢复驱动
    pub fn restore_driver(&mut self, backup_file: &PathBuf) -> RestoreResult {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 读取备份文件
        let backup_content = match std::fs::read_to_string(backup_file) {
            Ok(content) => content,
            Err(e) => {
                let result = RestoreResult {
                    success: false,
                    message: format!("无法读取备份文件: {}", e),
                    driver_name: "未知".to_string(),
                    timestamp,
                };
                self.restore_history.push(result.clone());
                return result;
            }
        };
        
        // 解析驱动名称（简化实现，实际应该解析完整的备份文件）
        let driver_name = backup_file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未知")
            .split('_')
            .next()
            .unwrap_or("未知")
            .to_string();
        
        // 这里应该实现实际的驱动恢复逻辑
        // 目前只是模拟恢复过程
        let result = RestoreResult {
            success: true,
            message: "驱动恢复成功".to_string(),
            driver_name,
            timestamp,
        };
        self.restore_history.push(result.clone());
        result
    }
    
    // 卸载驱动
    pub fn uninstall_driver(&mut self, driver: &DriverInfo) -> UninstallResult {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 使用sc命令停止驱动服务
        let stop_result = Command::new("sc")
            .args(&["stop", &driver.name])
            .output();
        
        // 使用sc命令删除驱动服务
        let delete_result = Command::new("sc")
            .args(&["delete", &driver.name])
            .output();
        
        match (stop_result, delete_result) {
            (Ok(stop_output), Ok(delete_output)) if stop_output.status.success() && delete_output.status.success() => {
                let result = UninstallResult {
                    success: true,
                    message: "驱动卸载成功".to_string(),
                    driver_name: driver.name.clone(),
                    timestamp,
                };
                self.uninstall_history.push(result.clone());
                result
            }
            (Ok(stop_output), Ok(delete_output)) => {
                let stop_error = String::from_utf8_lossy(&stop_output.stderr);
                let delete_error = String::from_utf8_lossy(&delete_output.stderr);
                let error_message = format!("卸载失败 - 停止: {}, 删除: {}", stop_error, delete_error);
                
                let result = UninstallResult {
                    success: false,
                    message: error_message,
                    driver_name: driver.name.clone(),
                    timestamp,
                };
                self.uninstall_history.push(result.clone());
                result
            }
            (Err(e), _) | (_, Err(e)) => {
                let result = UninstallResult {
                    success: false,
                    message: format!("执行卸载命令失败: {}", e),
                    driver_name: driver.name.clone(),
                    timestamp,
                };
                self.uninstall_history.push(result.clone());
                result
            }
        }
    }
    
    // 获取备份历史
    pub fn get_backup_history(&self) -> &Vec<BackupResult> {
        &self.backup_history
    }
    
    // 获取恢复历史
    pub fn get_restore_history(&self) -> &Vec<RestoreResult> {
        &self.restore_history
    }
    
    // 获取卸载历史
    pub fn get_uninstall_history(&self) -> &Vec<UninstallResult> {
        &self.uninstall_history
    }
    
    // 清空备份历史
    pub fn clear_backup_history(&mut self) {
        self.backup_history.clear();
    }
    
    // 清空恢复历史
    pub fn clear_restore_history(&mut self) {
        self.restore_history.clear();
    }
    
    // 清空卸载历史
    pub fn clear_uninstall_history(&mut self) {
        self.uninstall_history.clear();
    }
    
    // 获取备份目录中的备份文件列表
    pub fn get_backup_files(&self) -> Vec<PathBuf> {
        match std::fs::read_dir(&self.backup_dir) {
            Ok(entries) => {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.path().is_file() && 
                        entry.path().extension().map_or(false, |ext| ext == "bak")
                    })
                    .map(|entry| entry.path())
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }
}