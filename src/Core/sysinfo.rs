use std::collections::HashMap;
use wmi::{WMIConnection, COMLibrary};
use super::edid;

#[derive(Debug)]
pub struct SystemInfo {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub os_version_formatted: Option<String>,
    pub manufacturer: Option<String>,
    pub motherboard: Option<String>,
    pub cpu: Option<String>,
    pub memory_info: Vec<String>,
    pub disk_info: Vec<String>,
    pub gpu_info: Vec<String>,
    pub network_adapters: Vec<String>,
    pub monitor_info: Vec<String>,
}

impl SystemInfo {
    pub fn new() -> Result<Self, String> {
        let com_con = COMLibrary::new().map_err(|e| format!("COM initialization failed: {}", e))?;
        let wmi_con = WMIConnection::new(com_con.into()).map_err(|e| format!("WMI connection failed: {}", e))?;

        let os_info = Self::get_os_info(&wmi_con)?;
        let manufacturer = Self::get_manufacturer(&wmi_con)?;
        let motherboard = Self::get_motherboard(&wmi_con)?;
        let cpu = Self::get_cpu(&wmi_con)?;
        let memory_info = Self::get_memory_info(&wmi_con)?;
        let disk_info = Self::get_disk_info(&wmi_con)?;
        let gpu_info = Self::get_gpu_info(&wmi_con)?;
        let network_adapters = Self::get_network_adapters(&wmi_con)?;
        let monitor_info = Self::get_monitor_info(&wmi_con)?;

        Ok(SystemInfo {
            os_name: Some(os_info.0),
            os_version: Some(os_info.1),
            os_version_formatted: Some(os_info.2),
            manufacturer: Some(manufacturer),
            motherboard: Some(motherboard),
            cpu: Some(cpu),
            memory_info,
            disk_info,
            gpu_info,
            network_adapters,
            monitor_info,
        })
    }

    fn get_os_info(wmi_con: &WMIConnection) -> Result<(String, String, String), String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Caption, Version, BuildNumber FROM Win32_OperatingSystem")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(os) = results.first() {
            let os_name = os.get("Caption")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Unknown OS".to_string());

            let os_version = os.get("Version")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Unknown Version".to_string());

            let build_number = os.get("BuildNumber")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Unknown".to_string());

            Ok((os_name, os_version, build_number))
        } else {
            Ok(("Unknown OS".to_string(), "Unknown Version".to_string(), "Unknown".to_string()))
        }
    }

    fn get_manufacturer(wmi_con: &WMIConnection) -> Result<String, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Manufacturer FROM Win32_ComputerSystem")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(system) = results.first() {
            Ok(system.get("Manufacturer")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Unknown".to_string()))
        } else {
            Ok("Unknown".to_string())
        }
    }

    fn get_motherboard(wmi_con: &WMIConnection) -> Result<String, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Product FROM Win32_BaseBoard")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(board) = results.first() {
            Ok(board.get("Product")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Unknown".to_string()))
        } else {
            Ok("Unknown".to_string())
        }
    }

    fn get_cpu(wmi_con: &WMIConnection) -> Result<String, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name FROM Win32_Processor")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(cpu) = results.first() {
            Ok(cpu.get("Name")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "Unknown CPU".to_string()))
        } else {
            Ok("Unknown CPU".to_string())
        }
    }

    fn get_memory_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Capacity FROM Win32_PhysicalMemory")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut memory_info = Vec::new();
        let mut total_memory = 0u64;
        
        // 先计算总内存
        for memory in &results {
            if let Some(capacity) = memory.get("Capacity") {
                if let wmi::Variant::UI8(bytes) = capacity {
                    total_memory += *bytes;
                }
            }
        }
        
        if total_memory > 0 {
            let total_gb = (total_memory as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32;
            memory_info.push(format!("总内存: {} GB", total_gb));
            
            // 显示每个内存条的信息
            for (i, memory) in results.iter().enumerate() {
                if let Some(capacity) = memory.get("Capacity") {
                    if let wmi::Variant::UI8(bytes) = capacity {
                        let gb = (*bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32;
                        memory_info.push(format!("  内存条{}: {} GB", i + 1, gb));
                    }
                }
            }
        }

        if memory_info.is_empty() {
            memory_info.push("未知内存".to_string());
        }

        Ok(memory_info)
    }

    fn get_disk_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Size, Model FROM Win32_DiskDrive")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut disk_info = Vec::new();
        for disk in results {
            let mut info = String::new();
            
            if let Some(model) = disk.get("Model") {
                if let wmi::Variant::String(s) = model {
                    info.push_str(s);
                }
            }
            
            if let Some(size) = disk.get("Size") {
                if let wmi::Variant::UI8(bytes) = size {
                    let gb = (*bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32;
                    if !info.is_empty() {
                        info.push_str(" - ");
                    }
                    info.push_str(&format!("{} GB", gb));
                }
            }
            
            if info.is_empty() {
                info = "Unknown Disk".to_string();
            }
            
            disk_info.push(info);
        }

        if disk_info.is_empty() {
            disk_info.push("Unknown".to_string());
        }

        Ok(disk_info)
    }

    fn get_gpu_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name FROM Win32_VideoController WHERE Name != 'Microsoft Basic Display Adapter'")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut gpu_info = Vec::new();
        for gpu in results {
            if let Some(name) = gpu.get("Name") {
                if let wmi::Variant::String(s) = name {
                    gpu_info.push(s.clone());
                }
            }
        }

        if gpu_info.is_empty() {
            gpu_info.push("Unknown GPU".to_string());
        }

        Ok(gpu_info)
    }

    fn get_network_adapters(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name FROM Win32_NetworkAdapter WHERE PhysicalAdapter = TRUE")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut network_adapters = Vec::new();
        for adapter in results {
            if let Some(name) = adapter.get("Name") {
                if let wmi::Variant::String(s) = name {
                    network_adapters.push(s.clone());
                }
            }
        }

        if network_adapters.is_empty() {
            network_adapters.push("Unknown Network Adapter".to_string());
        }

        Ok(network_adapters)
    }

    fn get_monitor_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        // 使用EDID模块获取完整的显示器信息
        edid::get_complete_monitor_info(wmi_con)
    }

    pub fn get_windows_version_display(os_name: &str, os_version: &str) -> String {
        // 解析版本号，例如 "10.0.19045"
        let version_parts: Vec<&str> = os_version.split('.').collect();
        
        if version_parts.len() >= 3 {
            let major_version = version_parts[0];
            let minor_version = version_parts[1];
            let build_number = version_parts[2];
            
            // 根据操作系统名称和版本号确定Windows版本
            if os_name.to_lowercase().contains("windows 11") {
                // Windows 11版本检测
                match build_number {
                    "22000" => "21H2",  // Windows 11 21H2 (初始版本)
                    "22621" => "22H2",  // Windows 11 22H2
                    "22631" => "23H2",  // Windows 11 23H2
                    "26100" => "24H2",  // Windows 11 24H2
                    _ => {
                        // 根据构建号推断版本
                        if let Ok(build) = build_number.parse::<u32>() {
                            if build >= 26000 { "24H2" }
                            else if build >= 22600 { "23H2" }
                            else if build >= 22000 { "22H2" }
                            else { "21H2" }
                        } else {
                            "未知版本"
                        }
                    }
                }.to_string()
            } else if os_name.to_lowercase().contains("windows 10") {
                // Windows 10版本检测
                match build_number {
                    "10240" => "1507",  // Windows 10初始版本
                    "10586" => "1511",  // Windows 10十一月更新
                    "14393" => "1607",  // Windows 10周年更新
                    "15063" => "1703",  // Windows 10创作者更新
                    "16299" => "1709",  // Windows 10秋季创作者更新
                    "17134" => "1803",  // Windows 10 2018年4月更新
                    "17763" => "1809",  // Windows 10 2018年10月更新
                    "18362" => "1903",  // Windows 10 2019年5月更新
                    "18363" => "1909",  // Windows 10 2019年11月更新
                    "19041" => "2004",  // Windows 10 2020年5月更新
                    "19042" => "20H2",  // Windows 10 2020年10月更新
                    "19043" => "21H1",  // Windows 10 2021年5月更新
                    "19044" => "21H2",  // Windows 10 2021年11月更新
                    "19045" => "22H2",  // Windows 10 2022年10月更新
                    _ => {
                        // 根据构建号推断版本
                        if let Ok(build) = build_number.parse::<u32>() {
                            if build >= 19045 { "22H2" }
                            else if build >= 19044 { "21H2" }
                            else if build >= 19043 { "21H1" }
                            else if build >= 19042 { "20H2" }
                            else if build >= 19041 { "2004" }
                            else if build >= 18363 { "1909" }
                            else if build >= 18362 { "1903" }
                            else if build >= 17763 { "1809" }
                            else if build >= 17134 { "1803" }
                            else if build >= 16299 { "1709" }
                            else if build >= 15063 { "1703" }
                            else if build >= 14393 { "1607" }
                            else if build >= 10586 { "1511" }
                            else if build >= 10240 { "1507" }
                            else { "未知版本" }
                        } else {
                            "未知版本"
                        }
                    }
                }.to_string()
            } else {
                // 其他Windows版本
                format!("{}H{}", major_version, minor_version)
            }
        } else {
            "未知版本".to_string()
        }
    }
}