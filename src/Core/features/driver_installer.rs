use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallableDriver {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub manufacturer: String,
    pub file_path: PathBuf,
    pub supported_os: Vec<String>,
    pub signature_status: String,
    pub install_method: InstallMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallMethod {
    INF,
    EXE,
    MSI,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationResult {
    pub success: bool,
    pub message: String,
    pub driver_name: String,
    pub timestamp: String,
    pub log_file: Option<PathBuf>,
}

#[allow(dead_code)]
pub struct DriverInstaller {
    install_history: Vec<InstallationResult>,
    temp_dir: PathBuf,
}

impl DriverInstaller {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("hamster_drivers");
        
        // 确保临时目录存在
        if !temp_dir.exists() {
            std::fs::create_dir_all(&temp_dir).unwrap_or_else(|_| {
                eprintln!("无法创建临时目录: {:?}", temp_dir);
            });
        }
        
        Self {
            install_history: Vec::new(),
            temp_dir,
        }
    }
    
    pub fn scan_drivers_in_directory(&self, directory: &PathBuf) -> Result<Vec<InstallableDriver>, String> {
        let mut drivers = Vec::new();
        
        if !directory.exists() {
            return Err(format!("目录不存在: {:?}", directory));
        }
        
        // 扫描目录中的驱动文件
        for entry in std::fs::read_dir(directory)
            .map_err(|e| format!("无法读取目录: {}", e))? 
        {
            let entry = entry.map_err(|e| format!("无法读取目录项: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(driver) = self.analyze_driver_file(&path) {
                    drivers.push(driver);
                }
            }
        }
        
        Ok(drivers)
    }
    
    fn analyze_driver_file(&self, file_path: &PathBuf) -> Option<InstallableDriver> {
        let extension = file_path.extension()?.to_str()?;
        
        match extension.to_lowercase().as_str() {
            "inf" => {
                // 分析INF文件
                Some(InstallableDriver {
                    name: file_path.file_stem()?.to_str()?.to_string(),
                    display_name: file_path.file_stem()?.to_str()?.to_string(),
                    version: "未知".to_string(),
                    manufacturer: "未知".to_string(),
                    file_path: file_path.clone(),
                    supported_os: vec!["Windows".to_string()],
                    signature_status: self.check_signature(file_path).unwrap_or("未验证".to_string()),
                    install_method: InstallMethod::INF,
                })
            }
            "exe" | "msi" => {
                // 分析可执行安装程序
                Some(InstallableDriver {
                    name: file_path.file_stem()?.to_str()?.to_string(),
                    display_name: file_path.file_stem()?.to_str()?.to_string(),
                    version: "未知".to_string(),
                    manufacturer: "未知".to_string(),
                    file_path: file_path.clone(),
                    supported_os: vec!["Windows".to_string()],
                    signature_status: self.check_signature(file_path).unwrap_or("未验证".to_string()),
                    install_method: if extension == "exe" { InstallMethod::EXE } else { InstallMethod::MSI },
                })
            }
            _ => None,
        }
    }
    
    fn check_signature(&self, file_path: &PathBuf) -> Result<String, String> {
        // 使用PowerShell检查文件签名
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Get-AuthenticodeSignature -FilePath '{}' | Select-Object Status", 
                        file_path.display())
            ])
            .output()
            .map_err(|e| format!("执行签名检查失败: {}", e))?;
        
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("Valid") {
                Ok("有效签名".to_string())
            } else if output_str.contains("NotSigned") {
                Ok("未签名".to_string())
            } else {
                Ok("签名无效".to_string())
            }
        } else {
            Err("签名检查失败".to_string())
        }
    }
    
    pub fn install_driver(&mut self, driver: &InstallableDriver) -> InstallationResult {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        match &driver.install_method {
            InstallMethod::INF => self.install_inf_driver(driver, &timestamp),
            InstallMethod::EXE => self.install_exe_driver(driver, &timestamp),
            InstallMethod::MSI => self.install_msi_driver(driver, &timestamp),
            InstallMethod::Manual => InstallationResult {
                success: false,
                message: "手动安装方法需要用户交互".to_string(),
                driver_name: driver.name.clone(),
                timestamp,
                log_file: None,
            },
        }
    }
    
    fn install_inf_driver(&mut self, driver: &InstallableDriver, timestamp: &str) -> InstallationResult {
        // 使用pnputil安装INF驱动
        let output = Command::new("pnputil")
            .args(&["/add-driver", driver.file_path.to_str().unwrap(), "/install"])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let result = InstallationResult {
                    success: true,
                    message: "驱动安装成功".to_string(),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
            Ok(output) => {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                let result = InstallationResult {
                    success: false,
                    message: format!("驱动安装失败: {}", error_msg),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
            Err(e) => {
                let result = InstallationResult {
                    success: false,
                    message: format!("执行安装命令失败: {}", e),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
        }
    }
    
    fn install_exe_driver(&mut self, driver: &InstallableDriver, timestamp: &str) -> InstallationResult {
        // 运行可执行安装程序
        let output = Command::new(&driver.file_path)
            .arg("/S")  // 静默安装参数
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let result = InstallationResult {
                    success: true,
                    message: "驱动安装程序执行成功".to_string(),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
            Ok(output) => {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                let result = InstallationResult {
                    success: false,
                    message: format!("驱动安装失败: {}", error_msg),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
            Err(e) => {
                let result = InstallationResult {
                    success: false,
                    message: format!("执行安装程序失败: {}", e),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
        }
    }
    
    fn install_msi_driver(&mut self, driver: &InstallableDriver, timestamp: &str) -> InstallationResult {
        // 使用msiexec安装MSI包
        let output = Command::new("msiexec")
            .args(&["/i", driver.file_path.to_str().unwrap(), "/quiet", "/norestart"])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let result = InstallationResult {
                    success: true,
                    message: "MSI安装包执行成功".to_string(),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
            Ok(output) => {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                let result = InstallationResult {
                    success: false,
                    message: format!("MSI安装失败: {}", error_msg),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
            Err(e) => {
                let result = InstallationResult {
                    success: false,
                    message: format!("执行MSI安装失败: {}", e),
                    driver_name: driver.name.clone(),
                    timestamp: timestamp.to_string(),
                    log_file: None,
                };
                self.install_history.push(result.clone());
                result
            }
        }
    }
    
    pub fn get_installation_history(&self) -> &Vec<InstallationResult> {
        &self.install_history
    }
    
    pub fn clear_installation_history(&mut self) {
        self.install_history.clear();
    }
}