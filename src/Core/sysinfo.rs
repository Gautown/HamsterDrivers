use sysinfo::{System, Disks};
use windows::Win32::System::SystemInformation::*;
use wmi::WMIConnection;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_ComputerSystem {
    manufacturer: String,
    model: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_BaseBoard {
    manufacturer: String,
    product: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_PhysicalMemory {
    manufacturer: Option<String>,
    capacity: Option<u64>,
    speed: Option<u32>,
    part_number: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_DiskDrive {
    manufacturer: Option<String>,
    model: Option<String>,
    size: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_NetworkAdapter {
    name: Option<String>,
    description: Option<String>,
    manufacturer: Option<String>,
    adapter_type: Option<String>,
    net_connection_status: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_VideoController {
    name: Option<String>,
    description: Option<String>,
    adapter_ram: Option<u32>,
    video_mode_description: Option<String>,
    driver_version: Option<String>,
    manufacturer: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SysWin32_DesktopMonitor {
    name: Option<String>,
    screen_height: Option<u32>,
    screen_width: Option<u32>,
    monitor_manufacturer: Option<String>,
    monitor_description: Option<String>,
}

// 定义系统信息结构体
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub os_version_formatted: Option<String>,
    pub manufacturer: Option<String>,
    pub motherboard: Option<String>,
    pub cpu: Option<String>,
    pub memory_info: Vec<String>,
    pub disk_info: Vec<String>,
    pub network_adapters: Vec<String>,
    pub gpu_info: Vec<String>,
    pub monitor_info: Vec<String>,
}

impl SystemInfo {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os_name = System::name();
        let os_version = System::os_version();
        let os_version_formatted = if let Some(version) = &os_version {
            if let Some(v) = parse_windows_version(version) {
                let (year, h_version) = get_windows_year_h_version(v);
                let short_year = year % 100;
                Some(format!("{}H{}", short_year, h_version))
            } else {
                None
            }
        } else {
            None
        };

        let manufacturer = match get_manufacturer_info() {
            Ok(manufacturer) => Some(manufacturer),
            Err(_) => System::host_name(),
        };

        let motherboard = match get_motherboard_info() {
            Ok((manufacturer, model)) => {
                let chinese_manufacturer = convert_to_chinese_manufacturer(&manufacturer);
                Some(format!("{} {}", chinese_manufacturer, model))
            },
            Err(_) => Some("无法获取主板信息".to_string()),
        };

        let cpu = if !sys.cpus().is_empty() {
            let cpu = &sys.cpus()[0];
            let cpu_name = cpu.brand();
            let cores = sys.cpus().len();
            let frequency = cpu.frequency();
            let (manufacturer, model) = extract_cpu_manufacturer_and_model(cpu_name);
            Some(format!("{} {}，{} 核心，{} MHz", manufacturer, model, cores, frequency))
        } else {
            None
        };

        let memory_info = match get_memory_info() {
            Ok(memory_info) => {
                let mut info = Vec::new();
                for mem in memory_info.iter() {
                    let manufacturer = mem.manufacturer.as_deref().unwrap_or("Unknown");
                    let part_number = mem.part_number.as_deref().unwrap_or("Unknown");
                    let capacity_mb = mem.capacity.map(|c| c / (1024 * 1024)).unwrap_or(0);
                    let speed_mhz = mem.speed.unwrap_or(0);
                    info.push(format!("{} {}，{} MB，{} MHz", manufacturer, part_number, capacity_mb, speed_mhz));
                }
                info
            },
            Err(_) => {
                vec![format!("总容量 {} MB", sys.total_memory() / 1024 / 1024)]
            }
        };

        let disk_info = match get_disk_info() {
            Ok(disks_info) => {
                let mut info = Vec::new();
                for disk in disks_info.iter() {
                    let mut manufacturer = disk.manufacturer.as_deref().unwrap_or("Unknown");
                    let model = disk.model.as_deref().unwrap_or("Unknown");
                    let size_gb = disk.size.map(|s| s / (1024 * 1024 * 1024)).unwrap_or(0);
                    
                    if manufacturer == "(标准磁盘驱动器)" || manufacturer.starts_with("(标准磁盘驱动器)") {
                        manufacturer = "";
                    }
                    
                    if manufacturer.is_empty() {
                        info.push(format!("{}，{} GB", model, size_gb));
                    } else {
                        info.push(format!("{} {}，{} GB", manufacturer, model, size_gb));
                    }
                }
                info
            },
            Err(_) => {
                let mut info = Vec::new();
                info.push("无法获取磁盘信息".to_string());
                info
            }
        };

        let network_adapters = match get_network_adapters() {
            Ok(network_adapters) => {
                let mut adapters = Vec::new();
                if !network_adapters.is_empty() {
                    for adapter in network_adapters.iter() {
                        let manufacturer = adapter.manufacturer.as_deref().unwrap_or("Unknown");
                        let description = adapter.description.as_deref().unwrap_or("Unknown");
                        adapters.push(format!("{} {}", manufacturer, description));
                    }
                } else {
                    adapters.push("未找到网络适配器".to_string());
                }
                adapters
            },
            Err(_) => vec!["无法获取网络适配器信息".to_string()],
        };

        let gpu_info = match get_gpu_info_via_cmd() {
            Ok(gpu_names) => {
                if !gpu_names.is_empty() {
                    gpu_names
                } else {
                    vec!["未找到显卡信息".to_string()]
                }
            },
            Err(_) => vec!["无法获取显卡信息".to_string()],
        };

        let monitor_info = vec!["显示器信息暂不可用".to_string()];

        Ok(SystemInfo {
            os_name,
            os_version,
            os_version_formatted,
            manufacturer,
            motherboard,
            cpu,
            memory_info,
            disk_info,
            network_adapters,
            gpu_info,
            monitor_info,
        })
    }
}

// 获取磁盘信息
fn get_disk_info() -> Result<Vec<SysWin32_DiskDrive>, Box<dyn std::error::Error>> {
    let wmi_con = WMIConnection::new(wmi::COMLibrary::new()?)?;
    
    // 查询磁盘驱动器信息
    let results: Vec<SysWin32_DiskDrive> = wmi_con.raw_query("SELECT Manufacturer, Model, Size FROM Win32_DiskDrive")?;
    
    Ok(results)
}

// 获取内存信息
fn get_memory_info() -> Result<Vec<SysWin32_PhysicalMemory>, Box<dyn std::error::Error>> {
    let wmi_con = WMIConnection::new(wmi::COMLibrary::new()?)?;
    
    // 查询物理内存信息
    let results: Vec<SysWin32_PhysicalMemory> = wmi_con.raw_query("SELECT Manufacturer, Capacity, Speed, PartNumber FROM Win32_PhysicalMemory")?;
    
    Ok(results)
}

// 提取CPU制造商和型号
fn extract_cpu_manufacturer_and_model(cpu_brand: &str) -> (String, String) {
    let brand_lower = cpu_brand.to_lowercase();
    
    if brand_lower.contains("intel") {
        let model = cpu_brand.replace("Intel(R)", "").replace("Intel", "").trim().to_string();
        ("Intel".to_string(), model)
    } else if brand_lower.contains("amd") {
        let model = cpu_brand.replace("AMD", "").trim().to_string();
        ("AMD".to_string(), model)
    } else {
        // 如果既不是Intel也不是AMD，则尝试从品牌名中分离制造商和型号
        let parts: Vec<&str> = cpu_brand.split_whitespace().collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1..].join(" "))
        } else {
            ("Unknown".to_string(), cpu_brand.to_string())
        }
    }
}

// 获取网络适配器信息
fn get_network_adapters() -> Result<Vec<SysWin32_NetworkAdapter>, Box<dyn std::error::Error>> {
    let wmi_con = WMIConnection::new(wmi::COMLibrary::new()?)?;
    
    // 查询网络适配器信息，只获取启用的网络适配器
    let results: Vec<SysWin32_NetworkAdapter> = wmi_con.raw_query(
        "SELECT Name, Description, Manufacturer, AdapterType, NetConnectionStatus FROM Win32_NetworkAdapter WHERE NetEnabled=true"
    )?;
    
    Ok(results)
}

// 获取显卡信息 - 使用系统命令
fn get_gpu_info_via_cmd() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::process::Command;
    
    // 使用wmic命令获取显卡信息
    let output = Command::new("wmic")
        .args(&["path", "win32_videocontroller", "get", "name,adapterram,driverdate", "/format:list"])
        .output()?;

    if !output.status.success() {
        return Err(format!("WMIC command failed with status: {}", output.status).into());
    }
    
    let stdout = String::from_utf8(output.stdout)?;
    
    // 解析输出，提取显卡信息
    let mut gpus = Vec::new();
    let lines: Vec<&str> = stdout.lines().collect();
    
    for line in lines {
        if line.trim().starts_with("Name=") {
            let gpu_name = line.trim_start_matches("Name=").trim();
            if !gpu_name.is_empty() && gpu_name != "None" {
                gpus.push(gpu_name.to_string());
            }
        }
    }
    
    if gpus.is_empty() {
        // 尝试使用另一种方法
        let output = Command::new("powershell")
            .args(&["Get-WmiObject", "-Class", "Win32_VideoController", "|", "Select-Object", "Name,", "AdapterRAM", "|", "Format-List"])
            .output()?;
            
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            let _current_gpu = String::new();
            let lines: Vec<&str> = stdout.lines().collect();
            
            for line in lines {
                if line.starts_with("Name") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() > 1 {
                        let gpu_name = parts[1].trim();
                        if !gpu_name.is_empty() && gpu_name != "None" {
                            gpus.push(gpu_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    Ok(gpus)
}

// 获取显示器信息
fn get_monitor_info() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::process::Command;
    
    // 尝试使用PowerShell命令获取显示器信息
    // 先尝试获取即插即用ID来识别显示器
    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-WmiObject -Class Win32_DesktopMonitor | Select-Object PNPDeviceID | Format-Table -HideTableHeaders")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut monitors = Vec::new();
        
        for line in stdout.lines() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() && trimmed_line != "None" {
                // 从即插即用ID中提取显示器信息
                let pnp_id = trimmed_line;
                // 提取厂商和型号信息，例如从"MONITOR\GSM57C3\{4d3fc3c3-49c5-11d9-b10c-806e6ee7de36}"中提取GSM57C3
                if pnp_id.contains('\\') {
                    let parts: Vec<&str> = pnp_id.split('\\').collect();
                    if parts.len() >= 2 {
                        let model = parts[1];
                        if !model.is_empty() {
                            monitors.push(format!("显示器型号: {}", model));
                        }
                    }
                } else {
                    monitors.push(pnp_id.to_string());
                }
            }
        }
        
        if !monitors.is_empty() {
            return Ok(monitors);
        }
    }
    
    // 如果上面的命令失败，返回空列表而不是错误
    Ok(Vec::new())
}

// 将英文制造商名称转换为中文名称
fn convert_to_chinese_manufacturer(manufacturer: &str) -> String {
    let manufacturer_lower = manufacturer.to_lowercase();
    
    if manufacturer_lower.contains("gigabyte") {
        "技嘉".to_string()
    } else if manufacturer_lower.contains("asus") {
        "华硕".to_string()
    } else if manufacturer_lower.contains("msi") {
        "微星".to_string()
    } else if manufacturer_lower.contains("asrock") {
        "华擎".to_string()
    } else if manufacturer_lower.contains("biostar") {
        "梅捷".to_string()
    } else if manufacturer_lower.contains("colorful") || manufacturer_lower.contains("cvalley") {
        "七彩虹".to_string()
    } else if manufacturer_lower.contains("ecs") || manufacturer_lower.contains("elitegroup") {
        "精英".to_string()
    } else if manufacturer_lower.contains("foxconn") {
        "富士康".to_string()
    } else if manufacturer_lower.contains("hp") {
        "惠普".to_string()
    } else if manufacturer_lower.contains("dell") {
        "戴尔".to_string()
    } else if manufacturer_lower.contains("lenovo") {
        "联想".to_string()
    } else if manufacturer_lower.contains("acer") {
        "宏碁".to_string()
    } else if manufacturer_lower.contains("apple") {
        "苹果".to_string()
    } else if manufacturer_lower.contains("toshiba") {
        "东芝".to_string()
    } else if manufacturer_lower.contains("sony") {
        "索尼".to_string()
    } else if manufacturer_lower.contains("fujitsu") {
        "富士通".to_string()
    } else if manufacturer_lower.contains("alienware") {
        "外星人".to_string()
    } else if manufacturer_lower.contains("micro") && manufacturer_lower.contains("star") {
        "微星".to_string()
    } else if manufacturer_lower.contains("intel") {
        "英特尔".to_string()
    } else {
        // 如果找不到匹配的中文名称，返回原始名称
        manufacturer.to_string()
    }
}

// 获取主板信息
fn get_motherboard_info() -> Result<(String, String), Box<dyn std::error::Error>> {
    let wmi_con = WMIConnection::new(wmi::COMLibrary::new()?)?;
    
    // 查询主板信息
    let baseboard_results: Vec<SysWin32_BaseBoard> = wmi_con.raw_query("SELECT Manufacturer, Product FROM Win32_BaseBoard")?;
    
    if !baseboard_results.is_empty() {
        let base_board = &baseboard_results[0];
        let manufacturer = base_board.manufacturer.clone();
        let product = base_board.product.clone();
        return Ok((manufacturer, product));
    }
    
    // 如果获取失败，返回错误
    Err("无法获取主板信息".into())
}

// 获取制造商信息
fn get_manufacturer_info() -> Result<String, Box<dyn std::error::Error>> {
    let wmi_con = WMIConnection::new(wmi::COMLibrary::new()?)?;
    
    // 尝试获取计算机系统信息
    let results: Vec<SysWin32_ComputerSystem> = wmi_con.raw_query("SELECT Manufacturer, Model FROM Win32_ComputerSystem")?;
    
    if !results.is_empty() {
        let computer_system = &results[0];
        return Ok(computer_system.manufacturer.clone());
    }
    
    // 如果没有获取到计算机系统信息，尝试获取主板信息
    let baseboard_results: Vec<SysWin32_BaseBoard> = wmi_con.raw_query("SELECT Manufacturer, Product FROM Win32_BaseBoard")?;
    
    if !baseboard_results.is_empty() {
        let base_board = &baseboard_results[0];
        return Ok(base_board.manufacturer.clone());
    }
    
    // 如果都失败了，返回错误
    Err("无法获取制造商信息".into())
}

// 解析 Windows 版本号
fn parse_windows_version(version: &str) -> Option<u32> {
    // 期望格式类似于 "10 (19045)" 或其他版本号
    if let (Some(start), Some(end)) = (version.find('('), version.find(')')) {
        let inner = &version[start+1..end];
        inner.parse::<u32>().ok()
    } else {
        None
    }
}

// 根据 Windows 10/11 的版本号推断年份和H版本号
fn get_windows_year_h_version(build_number: u32) -> (u32, u32) {
    // Windows 10/11 主要版本对照表，转换为H*格式
    match build_number {
        10240 => (2015, 1),  // RTM - H1
        10586 => (2015, 2),  // November Update - H2
        14393 => (2016, 2),  // Anniversary Update - H2
        15063 => (2017, 1),  // Creators Update - H1
        16299 => (2017, 2),  // Fall Creators Update - H2
        17134 => (2018, 1),  // April 2018 Update - H1
        17763 => (2018, 2),  // October 2018 Update - H2
        18362 => (2019, 1),  // May 2019 Update - H1
        18363 => (2019, 2),  // November 2019 Update - H2
        19041 => (2020, 1),  // Version 2004 - H1
        19042 => (2020, 2),  // Version 20H2 - H2
        19043 => (2021, 1),  // Version 21H1 - H1
        19044 => (2021, 2),  // Version 21H2 - H2
        19045 => (2022, 1),  // Version 22H2 - H1 (Note: 22H2 is the 1st half of 2022)
        22000 => (2021, 2),  // Windows 11 21H2 - H2
        22621 => (2022, 2),  // Windows 11 22H2 - H2
        _ => {
            // 对于未知版本，根据版本号大致估算
            if build_number > 22000 {
                // 可能是 Windows 11
                if build_number >= 22621 {
                    (2022, 2)  // H2
                } else {
                    (2021, 2)  // H2
                }
            } else if build_number >= 19045 {
                (2022, 1)  // H1 (22H2)
            } else if build_number >= 19041 {
                (2020, 1)  // H1 (2004)
            } else {
                (2019, 1)  // H1 (默认值)
            }
        }
    }
}