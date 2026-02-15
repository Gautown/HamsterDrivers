use std::collections::HashMap;
use wmi::{WMIConnection, COMLibrary};
use hardware_query::HardwareInfo;
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
            .raw_query("SELECT Manufacturer, Capacity, MemoryType, Speed, SMBIOSMemoryType FROM Win32_PhysicalMemory")
            .map_err(|e| format!("WMI query失败: {}", e))?;

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
            
            // 显示每个内存条的详细信息
            for (i, memory) in results.iter().enumerate() {
                let mut mem_info = String::new();
                
                // 制造商
                let manufacturer = memory.get("Manufacturer")
                    .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("未知制造商")
                    .trim();
                
                // 容量
                let capacity_gb = memory.get("Capacity")
                    .and_then(|v| if let wmi::Variant::UI8(bytes) = v { 
                        Some((*bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32) 
                    } else { None })
                    .unwrap_or(0);
                
                // 内存代数 - 使用更全面的检测方法
                let generation = {
                    // 首先尝试 SMBIOSMemoryType（更准确）
                    let smbios_type_result: Option<&str> = memory.get("SMBIOSMemoryType")
                        .and_then(|v| {
                            if let wmi::Variant::UI4(mem_type) = v {
                                match mem_type {
                                    20 => Some("DDR"),
                                    21 => Some("DDR2"),
                                    22 => Some("DDR2 FB-DIMM"),
                                    24 => Some("DDR3"),
                                    26 => Some("DDR4"),
                                    29 => Some("DDR5"),
                                    _ => None
                                }
                            } else {
                                None
                            }
                        });
                    
                    // 如果 SMBIOS 类型可用，使用它
                    if let Some(gen) = smbios_type_result {
                        gen
                    } else {
                        // 否则回退到 MemoryType
                        memory.get("MemoryType")
                            .and_then(|v| {
                                if let wmi::Variant::UI4(mem_type) = v {
                                    match mem_type {
                                        20 => Some("DDR"),
                                        21 => Some("DDR2"),
                                        24 => Some("DDR3"),
                                        26 => Some("DDR4"),
                                        34 => Some("DDR5"),
                                        _ => Some("未知")
                                    }
                                } else {
                                    None
                                }
                            })
                            .unwrap_or("未知")
                    }
                };
                
                // 频率
                let speed = memory.get("Speed")
                    .and_then(|v| if let wmi::Variant::UI4(mhz) = v { Some(*mhz) } else { None })
                    .unwrap_or(0);
                
                // 格式化显示：内存n：制造商-容量-代数@频率
                mem_info.push_str(&format!("内存{}：{}-{}GB-{}@{}MHz", 
                    i + 1, manufacturer, capacity_gb, generation, speed));
                
                memory_info.push(mem_info);
            }
        }

        if memory_info.is_empty() {
            memory_info.push("未知内存".to_string());
        }

        Ok(memory_info)
    }

    fn get_disk_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Manufacturer, Model, Size, MediaType FROM Win32_DiskDrive")
            .map_err(|e| format!("WMI query失败: {}", e))?;

        let mut disk_info = Vec::new();
        for (i, disk) in results.iter().enumerate() {
            let mut info = String::new();
            
            // 制造商（去掉"(标准磁盘驱动器)"描述）
            let manufacturer = disk.get("Manufacturer")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.as_str()) } else { None })
                .map(|s| s.replace("(标准磁盘驱动器)", "").trim().to_string())
                .unwrap_or("未知制造商".to_string());
            
            // 型号
            let model = disk.get("Model")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("未知型号")
                .trim();
            
            // 容量
            let capacity_gb = disk.get("Size")
                .and_then(|v| if let wmi::Variant::UI8(bytes) = v { 
                    Some((*bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u32) 
                } else { None })
                .unwrap_or(0);
            
            // 硬盘类型判断（固态/机械/U盘）
            let disk_type = {
                // 首先检查MediaType
                let media_type = disk.get("MediaType")
                    .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.to_lowercase()) } else { None });
                
                // 检查型号名称中的关键词
                let model_lower = model.to_lowercase();
                
                // 固态硬盘判断逻辑（优先级最高）
                if media_type.as_deref() == Some("ssd") || 
                   media_type.as_deref() == Some("solid state drive") ||
                   media_type.as_deref() == Some("nvme") ||
                   model_lower.contains("ssd") ||
                   model_lower.contains("nvme") ||
                   model_lower.contains("solid state") ||
                   model_lower.contains("flash") ||
                   model_lower.contains("m.2") ||
                   model_lower.contains("m2") ||
                   model_lower.contains("pcie") ||
                   model_lower.contains("sata ssd") ||
                   model_lower.contains("sata-ssd") ||
                   // 常见SSD制造商品牌
                   model_lower.contains("samsung") && (model_lower.contains("evo") || model_lower.contains("pro") || model_lower.contains("qvo")) ||
                   model_lower.contains("crucial") ||
                   model_lower.contains("kingston") && model_lower.contains("ssd") ||
                   model_lower.contains("wd") && model_lower.contains("blue") ||
                   model_lower.contains("seagate") && model_lower.contains("firecuda") ||
                   model_lower.contains("intel") && model_lower.contains("ssd") {
                    "固态"
                } 
                // U盘判断逻辑
                else if media_type.as_deref() == Some("external hard disk media") ||
                          media_type.as_deref() == Some("removable media") ||
                          model_lower.contains("usb") ||
                          model_lower.contains("removable") ||
                          model_lower.contains("external") ||
                          model_lower.contains("flash drive") ||
                          model_lower.contains("pen drive") {
                    "U盘"
                } 
                // 机械硬盘判断逻辑
                else if media_type.as_deref() == Some("hdd") ||
                          media_type.as_deref() == Some("hard disk drive") ||
                          media_type.as_deref() == Some("fixed hard disk media") ||
                          model_lower.contains("hdd") ||
                          model_lower.contains("hard disk") ||
                          model_lower.contains("wd") && model_lower.contains("green") ||
                          model_lower.contains("wd") && model_lower.contains("black") ||
                          model_lower.contains("wd") && model_lower.contains("red") ||
                          model_lower.contains("wd") && model_lower.contains("purple") ||
                          model_lower.contains("seagate") && model_lower.contains("barracuda") ||
                          model_lower.contains("toshiba") && model_lower.contains("dt") {
                    "机械"
                } else {
                    // 默认根据接口类型判断
                    if model_lower.contains("usb") {
                        "U盘"
                    } else {
                        // 根据容量和型号特征智能判断
                        if capacity_gb <= 512 && (model_lower.contains("flash") || model_lower.contains("sd")) {
                            "U盘"
                        } else if capacity_gb >= 1000 && (model_lower.contains("wd") || model_lower.contains("seagate") || model_lower.contains("toshiba")) {
                            "机械"
                        } else {
                            "机械" // 默认认为是机械硬盘
                        }
                    }
                }
            };
            
            // 格式化显示：硬盘n：制造商-型号-容量-类型
            info.push_str(&format!("硬盘{}：{}-{}-{}GB-{}", 
                i + 1, manufacturer, model, capacity_gb, disk_type));
            
            disk_info.push(info);
        }

        if disk_info.is_empty() {
            disk_info.push("未知硬盘".to_string());
        }

        Ok(disk_info)
    }

    fn get_gpu_info(_wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let mut gpu_info = Vec::new();
        
        // 使用hardware-query库获取显卡信息
        match HardwareInfo::query() {
            Ok(hw_info) => {
                // 获取所有GPU信息
                let gpus = hw_info.gpus();
                
                for (i, gpu) in gpus.iter().enumerate() {
                    let mut gpu_info_str = String::new();
                    
                    // 获取显卡制造商和型号
                    let vendor = gpu.vendor();
                    let model_name = gpu.model_name();
                    
                    // 获取显存信息（GB），智能选择最准确的信息源
                    let vram_gb = {
                        let model_based_vram = Self::get_vram_by_model(&format!("{} {}", vendor, model_name)) as f64;
                        let library_vram = gpu.memory_gb();
                        
                        // 智能判断：如果型号匹配的显存与库返回的显存差异很大，且型号匹配的显存更合理，则使用型号匹配
                        let model_name_lower = model_name.to_lowercase();
                        
                        // 对于专业显卡（如RTX A4000），如果库返回的显存明显小于型号匹配的显存，则使用型号匹配
                        if (model_name_lower.contains("rtx a4000") && library_vram < 10.0) ||
                           (model_name_lower.contains("rtx") && library_vram < model_based_vram * 0.5) ||
                           (library_vram < 2.0 || library_vram > 100.0) {
                            model_based_vram
                        } else {
                            // 否则使用库提供的信息
                            library_vram
                        }
                    };
                    
                    // 格式化显示：显卡n：制造商+型号+显存
                    gpu_info_str.push_str(&format!("显卡{}：{}+{}+{}GB", i + 1, vendor, model_name, vram_gb));
                    
                    gpu_info.push(gpu_info_str);
                }
            }
            Err(e) => {
                // 如果hardware-query失败，回退到WMI查询
                let results: Vec<HashMap<String, wmi::Variant>> = _wmi_con
                    .raw_query("SELECT Name FROM Win32_VideoController WHERE Name != 'Microsoft Basic Display Adapter'")
                    .map_err(|e| format!("WMI query failed: {}", e))?;
                
                for (i, gpu) in results.iter().enumerate() {
                    let mut gpu_info_str = String::new();
                    
                    let name = gpu.get("Name")
                        .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("未知型号");
                    
                    let (manufacturer, model) = {
                        let name_lower = name.to_lowercase();
                        if name_lower.contains("nvidia") {
                            ("NVIDIA", name.trim())
                        } else if name_lower.contains("amd") || name_lower.contains("radeon") {
                            ("AMD", name.trim())
                        } else if name_lower.contains("intel") {
                            ("Intel", name.trim())
                        } else {
                            ("未知制造商", name.trim())
                        }
                    };
                    
                    let vram_gb = Self::get_vram_by_model(name);
                    
                    gpu_info_str.push_str(&format!("显卡{}：{}+{}+{}GB", i + 1, manufacturer, model, vram_gb));
                    gpu_info.push(gpu_info_str);
                }
            }
        }

        if gpu_info.is_empty() {
            gpu_info.push("未知显卡".to_string());
        }

        Ok(gpu_info)
    }
    
    // 辅助函数：根据显卡型号获取显存
    fn get_vram_by_model(name: &str) -> u32 {
        let name_lower = name.to_lowercase();
        
        // NVIDIA显卡
        if name_lower.contains("rtx a4000") {
            16 // RTX A4000 专业卡有16GB显存
        } else if name_lower.contains("rtx 4090") {
            24 // RTX 4090 24GB
        } else if name_lower.contains("rtx 4080") {
            16 // RTX 4080 16GB
        } else if name_lower.contains("rtx 4070") || name_lower.contains("rtx 4070 ti") {
            12 // RTX 4070/Ti 12GB
        } else if name_lower.contains("rtx 4060") || name_lower.contains("rtx 4060 ti") {
            8 // RTX 4060/Ti 8GB
        } else if name_lower.contains("rtx 4050") {
            6 // RTX 4050 6GB
        } else if name_lower.contains("rtx 3090") {
            24 // RTX 3090 24GB
        } else if name_lower.contains("rtx 3080") {
            10 // RTX 3080 10GB（部分12GB）
        } else if name_lower.contains("rtx 3070") || name_lower.contains("rtx 3070 ti") {
            8 // RTX 3070/Ti 8GB
        } else if name_lower.contains("rtx 3060") || name_lower.contains("rtx 3060 ti") {
            12 // RTX 3060 12GB / RTX 3060 Ti 8GB
        } else if name_lower.contains("rtx 3050") {
            8 // RTX 3050 8GB
        } else if name_lower.contains("gtx 1660") {
            6 // GTX 1660 6GB
        } else if name_lower.contains("gtx 1650") {
            4 // GTX 1650 4GB
        } 
        // AMD显卡
        else if name_lower.contains("radeon rx 7900 xtx") {
            24 // RX 7900 XTX 24GB
        } else if name_lower.contains("radeon rx 7900 xt") {
            20 // RX 7900 XT 20GB
        } else if name_lower.contains("radeon rx 7800 xt") {
            16 // RX 7800 XT 16GB
        } else if name_lower.contains("radeon rx 7700 xt") {
            12 // RX 7700 XT 12GB
        } else if name_lower.contains("radeon rx 7600") {
            8 // RX 7600 8GB
        } else if name_lower.contains("radeon rx 6950 xt") {
            16 // RX 6950 XT 16GB
        } else if name_lower.contains("radeon rx 6900 xt") {
            16 // RX 6900 XT 16GB
        } else if name_lower.contains("radeon rx 6800 xt") {
            16 // RX 6800 XT 16GB
        } else if name_lower.contains("radeon rx 6800") {
            16 // RX 6800 16GB
        } else if name_lower.contains("radeon rx 6700 xt") {
            12 // RX 6700 XT 12GB
        } else if name_lower.contains("radeon rx 6600 xt") {
            8 // RX 6600 XT 8GB
        } else if name_lower.contains("radeon rx 6600") {
            8 // RX 6600 8GB
        }
        // 集成显卡
        else if name_lower.contains("intel") || name_lower.contains("radeon graphics") || name_lower.contains("uhd graphics") {
            2 // 集成显卡通常共享内存，显示为2GB
        }
        // 默认值
        else {
            // 根据显卡名称中的关键词估算
            if name_lower.contains("rtx") || name_lower.contains("gtx") {
                if name_lower.contains("80") || name_lower.contains("90") {
                    16 // 80/90系列高端卡
                } else if name_lower.contains("70") {
                    8 // 70系列中高端卡
                } else if name_lower.contains("60") {
                    6 // 60系列中端卡
                } else if name_lower.contains("50") {
                    4 // 50系列入门卡
                } else {
                    6 // 默认NVIDIA游戏卡
                }
            } else if name_lower.contains("radeon") {
                if name_lower.contains("7900") || name_lower.contains("6900") || name_lower.contains("6800") {
                    16 // 高端AMD卡
                } else if name_lower.contains("7800") || name_lower.contains("7700") || name_lower.contains("6700") {
                    12 // 中高端AMD卡
                } else if name_lower.contains("7600") || name_lower.contains("6600") {
                    8 // 中端AMD卡
                } else {
                    4 // 默认AMD卡
                }
            } else {
                4 // 其他未知显卡默认值
            }
        }
    }

    fn get_network_adapters(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name, Manufacturer, Speed FROM Win32_NetworkAdapter WHERE PhysicalAdapter = TRUE")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut network_adapters = Vec::new();
        for adapter in results {
            let mut adapter_info = String::new();
            
            // 获取适配器名称
            let name = adapter.get("Name")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("未知型号");
            
            // 获取制造商
            let manufacturer = adapter.get("Manufacturer")
                .and_then(|v| if let wmi::Variant::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("未知制造商");
            
            // 获取速度（转换为Mbps，处理异常值）
            let speed_mbps = adapter.get("Speed")
                .and_then(|v| if let wmi::Variant::UI8(speed) = v { 
                    let mbps = (*speed as f64 / 1_000_000.0).round() as u32;
                    // 过滤异常值（如4294967295Mbps）
                    if mbps > 100000 || mbps == 0 { // 大于100Gbps或为0视为异常
                        None
                    } else {
                        Some(mbps)
                    }
                } else { None })
                .unwrap_or(0);
            
            // 判断适配器类型
            let adapter_type = {
                let name_lower = name.to_lowercase();
                if name_lower.contains("bluetooth") || name_lower.contains("蓝牙") {
                    "蓝牙"
                } else if name_lower.contains("wifi") || name_lower.contains("wireless") || 
                          name_lower.contains("wi-fi") || name_lower.contains("802.11") {
                    "WiFi"
                } else {
                    "网卡"
                }
            };
            
            // 格式化显示：蓝牙or网卡orWiFi：制造商-型号-速度
            adapter_info.push_str(&format!("{}：{}-{}-{}Mbps", adapter_type, manufacturer, name, speed_mbps));
            
            network_adapters.push(adapter_info);
        }

        if network_adapters.is_empty() {
            network_adapters.push("未知网络适配器".to_string());
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