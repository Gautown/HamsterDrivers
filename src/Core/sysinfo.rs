use wmi::{COMLibrary, WMIConnection};
use std::collections::HashMap;

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
    pub fn new() -> Result<Self, String> {
        let com_con = COMLibrary::new()
            .map_err(|e| format!("Failed to initialize COM library: {}", e))?;
        let wmi_con = WMIConnection::new(com_con.into())
            .map_err(|e| format!("Failed to create WMI connection: {}", e))?;

        let os_name = Self::get_os_name(&wmi_con)?;
        let os_version = Self::get_os_version(&wmi_con)?;
        let os_version_formatted = Self::get_os_version_formatted(&wmi_con)?;
        let manufacturer = Self::get_manufacturer(&wmi_con)?;
        let motherboard = Self::get_motherboard(&wmi_con)?;
        let cpu = Self::get_cpu(&wmi_con)?;
        let memory_info = Self::get_memory_info(&wmi_con)?;
        let disk_info = Self::get_disk_info(&wmi_con)?;
        let network_adapters = Self::get_network_adapters(&wmi_con)?;
        let gpu_info = Self::get_gpu_info(&wmi_con)?;
        let monitor_info = Self::get_monitor_info(&wmi_con)?;

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

    fn get_os_name(wmi_con: &WMIConnection) -> Result<Option<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Caption FROM Win32_OperatingSystem")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(os) = results.first() {
            if let Some(caption) = os.get("Caption") {
                if let wmi::Variant::String(caption_str) = caption {
                    return Ok(Some(caption_str.replace("Microsoft ", "").trim().to_string()));
                }
            }
        }
        Ok(None)
    }

    fn get_os_version(wmi_con: &WMIConnection) -> Result<Option<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Version FROM Win32_OperatingSystem")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(os) = results.first() {
            if let Some(version) = os.get("Version") {
                if let wmi::Variant::String(version_str) = version {
                    return Ok(Some(version_str.clone()));
                }
            }
        }
        Ok(None)
    }

    fn get_os_version_formatted(wmi_con: &WMIConnection) -> Result<Option<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Version, BuildNumber FROM Win32_OperatingSystem")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(os) = results.first() {
            let mut version_parts = Vec::new();

            if let Some(version) = os.get("Version") {
                if let wmi::Variant::String(version_str) = version {
                    version_parts.push(version_str.clone());
                }
            }

            if let Some(build) = os.get("BuildNumber") {
                if let wmi::Variant::String(build_str) = build {
                    version_parts.push(format!("(Build {})", build_str));
                }
            }

            if !version_parts.is_empty() {
                return Ok(Some(version_parts.join(" ")));
            }
        }
        Ok(None)
    }

    fn get_manufacturer(wmi_con: &WMIConnection) -> Result<Option<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Manufacturer FROM Win32_ComputerSystem")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(system) = results.first() {
            if let Some(manufacturer) = system.get("Manufacturer") {
                if let wmi::Variant::String(manufacturer_str) = manufacturer {
                    return Ok(Some(manufacturer_str.clone()));
                }
            }
        }
        Ok(None)
    }

    fn get_motherboard(wmi_con: &WMIConnection) -> Result<Option<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Product, Manufacturer FROM Win32_BaseBoard")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(board) = results.first() {
            let mut parts = Vec::new();

            if let Some(manufacturer) = board.get("Manufacturer") {
                if let wmi::Variant::String(mfg) = manufacturer {
                    parts.push(mfg.clone());
                }
            }

            if let Some(product) = board.get("Product") {
                if let wmi::Variant::String(prod) = product {
                    parts.push(prod.clone());
                }
            }

            if !parts.is_empty() {
                return Ok(Some(parts.join(" ")));
            }
        }
        Ok(None)
    }

    fn get_cpu(wmi_con: &WMIConnection) -> Result<Option<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name FROM Win32_Processor")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        if let Some(cpu) = results.first() {
            if let Some(name) = cpu.get("Name") {
                if let wmi::Variant::String(name_str) = name {
                    return Ok(Some(name_str.clone()));
                }
            }
        }
        Ok(None)
    }

    // 根据SMBIOSMemoryType映射到DDR代数（基于SMBIOS标准）
    fn get_ddr_generation_from_smbios(smbios_type: u16) -> String {
        match smbios_type {
            18 => "DDR".to_string(),      // SDRAM
            20 => "DDR".to_string(),      // DDR
            21 => "DDR2".to_string(),     // DDR2
            22 => "DDR2 FB-DIMM".to_string(), // DDR2 FB-DIMM
            24 => "DDR3".to_string(),     // DDR3
            26 => "DDR4".to_string(),     // DDR4
            28 => "DDR4".to_string(),     // LPDDR4
            29 => "DDR5".to_string(),     // DDR5
            30 => "LPDDR5".to_string(),   // LPDDR5
            31 => "HBM2".to_string(),     // HBM2
            32 => "HBM3".to_string(),     // HBM3
            34 => "DDR5".to_string(),     // LPDDR5X
            // 其他类型映射
            17 => "SDRAM".to_string(),    // SDRAM
            19 => "EDO".to_string(),      // EDO
            23 => "DDR3".to_string(),     // DDR3 FB-DIMM
            25 => "FBD2".to_string(),     // FBD2
            27 => "LPDDR3".to_string(),   // LPDDR3
            33 => "DDR5".to_string(),     // DDR5 NVDIMM-P
            _ => "".to_string(),          // 未知类型
        }
    }

    // 根据内存速度推断DDR代数（备用方案）
    fn get_ddr_generation_from_speed(speed: u32) -> String {
        match speed {
            200..=399 => "DDR".to_string(),      // DDR: 200-399MHz
            400..=799 => "DDR2".to_string(),     // DDR2: 400-799MHz
            800..=1599 => "DDR3".to_string(),    // DDR3: 800-1599MHz
            1600..=3199 => "DDR4".to_string(),   // DDR4: 1600-3199MHz
            3200..=6400 => "DDR5".to_string(),   // DDR5: 3200-6400MHz
            _ => "".to_string(),                 // 未知或超出范围
        }
    }

    fn get_memory_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Manufacturer, Capacity, Speed, SMBIOSMemoryType FROM Win32_PhysicalMemory")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut memory_info = Vec::new();

        for (index, mem) in results.iter().enumerate() {
            let mut info_parts = Vec::new();
            
            // 添加内存序号
            let memory_number = index + 1;

            // 首先添加制造商（品牌）
            if let Some(manufacturer) = mem.get("Manufacturer") {
                if let wmi::Variant::String(mfg) = manufacturer {
                    info_parts.push(mfg.clone());  // 品牌
                }
            }

            // 然后添加容量
            if let Some(capacity) = mem.get("Capacity") {
                match capacity {
                    wmi::Variant::String(capacity_str) => {
                        if let Ok(capacity_bytes) = capacity_str.parse::<u64>() {
                            let capacity_gb = capacity_bytes / (1024 * 1024 * 1024);
                            if capacity_gb > 0 {
                                info_parts.push(format!("{}GB", capacity_gb));  // 容量
                            }
                        }
                    },
                    wmi::Variant::UI8(capacity_bytes) => {
                        let capacity_gb = capacity_bytes / (1024 * 1024 * 1024);
                        if capacity_gb > 0 {
                            info_parts.push(format!("{}GB", capacity_gb));
                        }
                    },
                    wmi::Variant::UI4(capacity_bytes) => {
                        let capacity_gb = *capacity_bytes as u64 / (1024 * 1024 * 1024);
                        if capacity_gb > 0 {
                            info_parts.push(format!("{}GB", capacity_gb));
                        }
                    },
                    _ => {}
                }
            }

            // 最后添加速度（主频）
            let mut memory_speed = 0;
            if let Some(speed) = mem.get("Speed") {
                if let wmi::Variant::UI4(speed_val) = speed {
                    memory_speed = *speed_val;
                    info_parts.push(format!("{}MHz", speed_val));  // 主频
                }
            }

            // 根据SMBIOSMemoryType获取DDR代数（优先使用）
            let mut ddr_generation = String::new();
            if let Some(smbios_type) = mem.get("SMBIOSMemoryType") {
                if let wmi::Variant::UI2(smbios_val) = smbios_type {
                    ddr_generation = Self::get_ddr_generation_from_smbios(*smbios_val);
                }
            }
            
            // 如果SMBIOSMemoryType不可用，则根据速度推断
            if ddr_generation.is_empty() && memory_speed > 0 {
                ddr_generation = Self::get_ddr_generation_from_speed(memory_speed);
            }
            
            if !ddr_generation.is_empty() {
                info_parts.push(ddr_generation);  // DDR代数
            }

            if !info_parts.is_empty() {
                // 格式：内存N：品牌-容量-主频-DDR
                memory_info.push(format!("内存{}：{}", memory_number, info_parts.join("-")));
            }
        }

        if memory_info.is_empty() {
            memory_info.push("Unknown Memory Information".to_string());
        }

        Ok(memory_info)
    }

    fn get_disk_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        // 尝试不同的查询方式，确保检测到所有硬盘
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Manufacturer, Model, Size, MediaType, InterfaceType FROM Win32_DiskDrive")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        println!("硬盘WMI查询结果数量: {}", results.len()); // 调试信息

        let mut disk_info = Vec::new();

        for (index, disk) in results.iter().enumerate() {
            println!("处理硬盘 {}: {:?}", index + 1, disk); // 调试信息
            
            let mut info_parts = Vec::new();

            // 首先添加硬盘序号
            let disk_number = index + 1;
            
            // 然后添加制造商（品牌）
            let mut brand_found = false;
            if let Some(manufacturer) = disk.get("Manufacturer") {
                if let wmi::Variant::String(manufacturer_str) = manufacturer {
                    if !manufacturer_str.trim().is_empty() && manufacturer_str != "(标准磁盘驱动器)" {
                        info_parts.push(manufacturer_str.clone());  // 品牌
                        brand_found = true;
                    }
                }
            }
            
            // 如果Manufacturer字段无效，尝试从Model中提取品牌
            if !brand_found {
                if let Some(model) = disk.get("Model") {
                    if let wmi::Variant::String(model_str) = model {
                        // 尝试从型号中提取常见品牌
                        let model_upper = model_str.to_uppercase();
                        if model_upper.contains("SEAGATE") {
                            info_parts.push("Seagate".to_string());
                            brand_found = true;
                        } else if model_upper.contains("WESTERN") || model_upper.contains("WD") {
                            info_parts.push("Western Digital".to_string());
                            brand_found = true;
                        } else if model_upper.contains("TOSHIBA") {
                            info_parts.push("Toshiba".to_string());
                            brand_found = true;
                        } else if model_upper.contains("SAMSUNG") {
                            info_parts.push("Samsung".to_string());
                            brand_found = true;
                        } else if model_upper.contains("HITACHI") {
                            info_parts.push("Hitachi".to_string());
                            brand_found = true;
                        } else if model_upper.contains("KINGSTON") {
                            info_parts.push("Kingston".to_string());
                            brand_found = true;
                        } else if model_upper.contains("GLOWAY") {
                            info_parts.push("Gloway".to_string());
                            brand_found = true;
                        } else if model_upper.contains("SANDISK") {
                            info_parts.push("SanDisk".to_string());
                            brand_found = true;
                        } else if model_upper.contains("CRUCIAL") {
                            info_parts.push("Crucial".to_string());
                            brand_found = true;
                        } else if model_upper.contains("ADATA") {
                            info_parts.push("ADATA".to_string());
                            brand_found = true;
                        }
                    }
                }
            }

            // 然后添加型号
            if let Some(model) = disk.get("Model") {
                if let wmi::Variant::String(model_str) = model {
                    info_parts.push(model_str.clone());  // 型号
                }
            }

            // 最后添加容量
            if let Some(size) = disk.get("Size") {
                match size {
                    wmi::Variant::String(size_str) => {
                        if let Ok(size_bytes) = size_str.parse::<u64>() {
                            let size_gb = size_bytes / (1024 * 1024 * 1024);
                            if size_gb > 0 {
                                info_parts.push(format!("{}GB", size_gb));  // 容量
                            }
                        }
                    },
                    wmi::Variant::UI8(size_bytes) => {
                        let size_gb = size_bytes / (1024 * 1024 * 1024);
                        if size_gb > 0 {
                            info_parts.push(format!("{}GB", size_gb));
                        }
                    },
                    wmi::Variant::UI4(size_bytes) => {
                        let size_gb = *size_bytes as u64 / (1024 * 1024 * 1024);
                        if size_gb > 0 {
                            info_parts.push(format!("{}GB", size_gb));
                        }
                    },
                    _ => {}
                }
            }

            if !info_parts.is_empty() {
                // 更准确地判断硬盘类型：HDD或SSD
                let disk_type = if let Some(model) = disk.get("Model") {
                    if let wmi::Variant::String(model_str) = model {
                        let model_upper = model_str.to_uppercase();
                        
                        // 更精确的SSD识别
                        if model_upper.contains("SSD") || 
                           model_upper.contains("SOLID") || 
                           model_upper.contains("FLASH") ||
                           model_upper.contains("NVME") ||
                           model_upper.contains("M.2") {
                            "SSD"
                        } 
                        // 更精确的HDD识别
                        else if model_upper.contains("HDD") || 
                                model_upper.contains("HARD DISK") || 
                                model_upper.contains("DISK DRIVE") ||
                                model_upper.contains("ST") && model_upper.contains("DM") ||  // Seagate硬盘
                                model_upper.contains("WD") ||  // Western Digital硬盘
                                model_upper.contains("TOSHIBA") && !model_upper.contains("SSD") {
                            "HDD"
                        }
                        // 根据特定品牌和型号判断
                        else if model_upper.contains("SAMSUNG") && model_upper.contains("EVO") {
                            "SSD"
                        }
                        else if model_upper.contains("GLOWAY") && model_upper.contains("PRO") {
                            "SSD"
                        }
                        // 默认使用更保守的判断
                        else {
                            // 根据容量和转速特征判断（SSD通常容量较小，HDD容量较大）
                            if let Some(size) = disk.get("Size") {
                                match size {
                                    wmi::Variant::UI8(size_bytes) => {
                                        let size_gb = size_bytes / (1024 * 1024 * 1024);
                                        // 如果容量小于500GB，更可能是SSD；大于1TB更可能是HDD
                                        if size_gb < 500 { "SSD" } else { "HDD" }
                                    },
                                    wmi::Variant::UI4(size_bytes) => {
                                        let size_gb = *size_bytes as u64 / (1024 * 1024 * 1024);
                                        if size_gb < 500 { "SSD" } else { "HDD" }
                                    },
                                    _ => "HDD"
                                }
                            } else {
                                "HDD"
                            }
                        }
                    } else {
                        "HDD"
                    }
                } else {
                    "HDD"
                };
                
                // 格式：硬盘N：品牌+型号+容量+类型
                disk_info.push(format!("硬盘{}：{} {}", disk_number, info_parts.join(" "), disk_type));
            }
        }

        if disk_info.is_empty() {
            disk_info.push("Unknown Disk Information".to_string());
        }

        Ok(disk_info)
    }

    fn get_network_adapters(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name, Description FROM Win32_NetworkAdapter WHERE NetEnabled = TRUE")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut adapters = Vec::new();

        for adapter in results.iter().take(5) { // Limit to first 5 adapters
            let mut info_parts = Vec::new();

            if let Some(description) = adapter.get("Description") {
                if let wmi::Variant::String(desc) = description {
                    info_parts.push(desc.clone());
                }
            }

            if !info_parts.is_empty() {
                adapters.push(info_parts.join(", "));
            }
        }

        if adapters.is_empty() {
            adapters.push("No Active Network Adapters".to_string());
        }

        Ok(adapters)
    }

    fn get_gpu_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name, AdapterRAM FROM Win32_VideoController")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut gpu_info = Vec::new();

        for gpu in results {
            let mut info_parts = Vec::new();

            if let Some(name) = gpu.get("Name") {
                if let wmi::Variant::String(name_str) = name {
                    info_parts.push(name_str.clone());
                }
            }

            if let Some(ram) = gpu.get("AdapterRAM") {
                if let wmi::Variant::UI4(ram_val) = ram {
                    let ram_mb = ram_val / (1024 * 1024);
                    info_parts.push(format!("{}MB", ram_mb));
                }
            }

            if !info_parts.is_empty() {
                gpu_info.push(info_parts.join(", "));
            }
        }

        if gpu_info.is_empty() {
            gpu_info.push("Unknown GPU Information".to_string());
        }

        Ok(gpu_info)
    }

    fn get_monitor_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let mut monitor_info = Vec::new();

        // 首先尝试使用WmiMonitorID来获取制造商和型号信息（这应该提供更具体的品牌和型号）
        // Note: WmiMonitorID might not be available on all systems, so we handle the error gracefully
        if let Ok(monitor_id_results) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
            "SELECT InstanceName, ManufacturerName, ProductCodeID, UserFriendlyName, SerialNumberID FROM WmiMonitorID WHERE Active=TRUE"
        ) {
            for monitor in monitor_id_results {
                let mut info_parts = Vec::new();

                // 处理制造商名称
                if let Some(manufacturer_raw) = monitor.get("ManufacturerName") {
                    if let wmi::Variant::Array(bytes) = manufacturer_raw {
                        let mut manufacturer_bytes = Vec::new();
                        for byte_variant in bytes {
                            // Try both UI1 (unsigned 8-bit) and I8 (signed 8-bit) variants
                            if let wmi::Variant::UI1(b) = byte_variant {
                                if *b != 0 { 
                                    manufacturer_bytes.push(*b);
                                }
                            } else if let wmi::Variant::I8(b) = byte_variant {
                                if *b != 0 { 
                                    manufacturer_bytes.push(*b as u8);
                                }
                            }
                        }
                        if !manufacturer_bytes.is_empty() {
                            if let Ok(manufacturer) = String::from_utf8(manufacturer_bytes) {
                                if !manufacturer.trim().is_empty() && manufacturer != "0000" {
                                    info_parts.push(manufacturer.trim().to_string());
                                }
                            }
                        }
                    }
                }

                // 处理产品代码ID（通常代表型号）
                if let Some(product_raw) = monitor.get("ProductCodeID") {
                    if let wmi::Variant::Array(bytes) = product_raw {
                        let mut product_bytes = Vec::new();
                        for byte_variant in bytes {
                            // Try both UI1 (unsigned 8-bit) and I8 (signed 8-bit) variants
                            if let wmi::Variant::UI1(b) = byte_variant {
                                if *b != 0 { 
                                    product_bytes.push(*b);
                                }
                            } else if let wmi::Variant::I8(b) = byte_variant {
                                if *b != 0 { 
                                    product_bytes.push(*b as u8);
                                }
                            }
                        }
                        if !product_bytes.is_empty() {
                            if let Ok(product) = String::from_utf8(product_bytes) {
                                if !product.trim().is_empty() {
                                    info_parts.push(product.trim().to_string());
                                }
                            }
                        }
                    }
                }

                // 处理用户友好名称
                if let Some(friendly_name_raw) = monitor.get("UserFriendlyName") {
                    if let wmi::Variant::Array(bytes) = friendly_name_raw {
                        let mut friendly_name_bytes = Vec::new();
                        for byte_variant in bytes {
                            // Try both UI1 (unsigned 8-bit) and I8 (signed 8-bit) variants
                            if let wmi::Variant::UI1(b) = byte_variant {
                                if *b != 0 { 
                                    friendly_name_bytes.push(*b);
                                }
                            } else if let wmi::Variant::I8(b) = byte_variant {
                                if *b != 0 { 
                                    friendly_name_bytes.push(*b as u8);
                                }
                            }
                        }
                        if !friendly_name_bytes.is_empty() {
                            if let Ok(friendly_name) = String::from_utf8(friendly_name_bytes) {
                                if !friendly_name.trim().is_empty() {
                                    info_parts.push(friendly_name.trim().to_string());
                                }
                            }
                        }
                    }
                }

                // 处理序列号（如果可用）
                if let Some(serial_raw) = monitor.get("SerialNumberID") {
                    if let wmi::Variant::Array(bytes) = serial_raw {
                        let mut serial_bytes = Vec::new();
                        for byte_variant in bytes {
                            // Try both UI1 (unsigned 8-bit) and I8 (signed 8-bit) variants
                            if let wmi::Variant::UI1(b) = byte_variant {
                                if *b != 0 { 
                                    serial_bytes.push(*b);
                                }
                            } else if let wmi::Variant::I8(b) = byte_variant {
                                if *b != 0 { 
                                    serial_bytes.push(*b as u8);
                                }
                            }
                        }
                        if !serial_bytes.is_empty() {
                            if let Ok(serial) = String::from_utf8(serial_bytes) {
                                if !serial.trim().is_empty() {
                                    info_parts.push(format!("S/N: {}", serial.trim()));
                                }
                            }
                        }
                    }
                }

                if !info_parts.is_empty() {
                    monitor_info.push(info_parts.join(" "));
                }
            }
        }

        // 如果WmiMonitorID没有返回有用的信息，则尝试其他方法
        if monitor_info.is_empty() {
            // 尝试使用Win32_DesktopMonitor获取显示器信息，同时尝试 to get more specific info
            let desktop_results: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT Name, MonitorType, PNPDeviceID, ScreenHeight, ScreenWidth FROM Win32_DesktopMonitor")
                .map_err(|e| format!("WMI query failed: {}", e))?;

            for monitor in desktop_results {
                let mut info_parts = Vec::new();

                // 不再跳过通用监视器，而是尝试从PNPDeviceID中提取信息
                let mut has_useful_info = false;
                
                if let Some(name) = monitor.get("Name") {
                    if let wmi::Variant::String(name_str) = name {
                        // 包含名称，但标记是否为通用名称
                        if name_str != "Generic PnP Monitor" && name_str != "Default Monitor" {
                            info_parts.push(name_str.clone());
                            has_useful_info = true;
                        }
                    }
                }

                if let Some(monitor_type) = monitor.get("MonitorType") {
                    if let wmi::Variant::String(type_str) = monitor_type {
                        // 包含类型，但标记是否为通用类型
                        if type_str != "默认监视器" && type_str != "通用即插即用监视器" && 
                           type_str != "Generic PnP Monitor" && type_str != "Default Monitor" {
                            info_parts.push(type_str.clone());
                            has_useful_info = true;
                        }
                    }
                }

                // 尝试从PNPDeviceID中提取制造商和型号信息（这是最重要的部分）
                if let Some(pnp_device_id) = monitor.get("PNPDeviceID") {
                    if let wmi::Variant::String(pnp_str) = pnp_device_id {
                        // 解析PNP设备ID以提取制造商和型号
                        // 常见格式: DISPLAY\<VendorModel>\<InstanceId> 或 MONITOR\<VendorModel>&<ProductID>&<OtherIDs>\<InstanceID>
                        let parts: Vec<&str> = pnp_str.split('\\').collect();
                        
                        for (index, part) in parts.iter().enumerate() {
                            // 第一部分通常是设备类型 (如 DISPLAY 或 MONITOR)，跳过
                            // 第二部分通常是厂商型号信息
                            if index == 1 && part.len() >= 3 {
                                let vendor_model = part.to_uppercase();
                                
                                // 尝试识别常见的厂商代码前缀
                                if vendor_model.len() >= 3 {
                                    let vendor_prefix = &vendor_model[..3];
                                    let vendor_name = match vendor_prefix {
                                        "SAM" => "Samsung",      // Samsung
                                        "DEL" => "Dell",         // Dell
                                        "APP" => "Apple",        // Apple
                                        "HWP" => "HP",           // HP
                                        "LEN" => "Lenovo",       // Lenovo
                                        "ACR" => "Acer",         // Acer
                                        "ASU" => "ASUS",         // ASUS
                                        "LGD" => "LG",           // LG
                                        "BNQ" => "BenQ",         // BenQ
                                        "IVM" => "IBM",          // IBM
                                        "NEC" => "NEC",          // NEC
                                        "CMO" => "Chi Mei",      // Chi Mei
                                        "AUO" => "AU Optronics", // AU Optronics
                                        "HSD" => "Hannspree",    // Hannspree
                                        "XMI" => "Xiaomi",       // Xiaomi
                                        "AOC" => "AOC",          // AOC
                                        "PHL" => "Philips",      // Philips
                                        "ACI" => "ACI",          // ACI
                                        "CPL" => "Compal",       // Compal
                                        "CPQ" => "Compaq",       // Compaq
                                        "DPC" => "Delta",        // Delta
                                        "DTK" => "DTK",          // DTK
                                        "FCM" => "Funai",        // Funai
                                        "GSM" => "Goldstar",     // Goldstar
                                        "HTC" => "HTC",          // HTC
                                        "ICL" => "Fujitsu ICL",  // Fujitsu ICL
                                        "IFS" => "InFocus",      // InFocus
                                        "KDS" => "KDS",          // KDS
                                        "LPL" => "LG Philips",   // LG Philips
                                        "LKM" => "ADLAS / Aladdin", // ADLAS / Aladdin
                                        "MLC" => "MediaQ",       // MediaQ
                                        "MS_" => "Panasonic",    // Panasonic
                                        "NOK" => "Nokia",        // Nokia
                                        "NVD" => "Nvidia",       // Nvidia
                                        "OPT" => "Optoma",       // Optoma
                                        "PRT" => "Parrot",       // Parrot
                                        "REL" => "Relisys",      // Relisys
                                        "SAN" => "Samsung",      // Samsung
                                        "SGI" => "SGI",          // SGI
                                        "SNY" => "Sony",         // Sony
                                        "SRC" => "Shamrock",     // Shamrock
                                        "TOS" => "Toshiba",      // Toshiba
                                        "TSB" => "Toshiba",      // Toshiba
                                        "VSC" => "ViewSonic",    // ViewSonic
                                        _ => {
                                            // 如果无法识别，尝试 to see if it looks like a vendor code (3-4 letters)
                                            if vendor_model.chars().all(|c| c.is_ascii_alphabetic()) && 
                                               (vendor_model.len() == 3 || vendor_model.len() == 4) {
                                                &vendor_model[..std::cmp::min(vendor_model.len(), 4)]
                                            } else {
                                                continue; // Skip if not a recognizable vendor code
                                            }
                                        }
                                    };
                                    
                                    // Extract model part (the rest after the vendor prefix)
                                    let model_part = if vendor_model.len() > 3 {
                                        &vendor_model[3..]
                                    } else {
                                        ""
                                    };
                                    
                                    // Add vendor name if not already present
                                    if !info_parts.contains(&vendor_name.to_string()) {
                                        info_parts.insert(0, vendor_name.to_string()); // 插入到开头作为品牌
                                        has_useful_info = true;
                                    }
                                    
                                    // Add model if present and not already added
                                    if !model_part.is_empty() && !info_parts.contains(&format!("Model: {}", model_part)) {
                                        info_parts.push(format!("Model: {}", model_part));
                                        has_useful_info = true;
                                    }
                                }
                            }
                            
                            // Also check for VEN_/DEV_ format in other parts
                            if part.contains("&") {
                                let sub_parts: Vec<&str> = part.split('&').collect();
                                
                                // 尝试解析供应商代码 (通常以VEN_开头)
                                for sub_part in &sub_parts {
                                    if sub_part.starts_with("VEN_") && sub_part.len() >= 8 {
                                        let vendor_code = &sub_part[4..8]; // 取4个字符的供应商代码
                                        
                                        // 将标准供应商代码转换为品牌名称
                                        let vendor_name = match vendor_code.to_uppercase().as_str() {
                                            "SAM" => "Samsung",
                                            "DEL" => "Dell",
                                            "APP" => "Apple",
                                            "HWP" => "HP",
                                            "LEN" => "Lenovo",
                                            "ACR" => "Acer",
                                            "ASU" => "ASUS",
                                            "LGD" => "LG",
                                            "BNQ" => "BenQ",
                                            "IVM" => "IBM",
                                            "NEC" => "NEC",
                                            "CMO" => "Chi Mei",
                                            "AUO" => "AU Optronics",
                                            "HSD" => "Hannspree",
                                            _ => vendor_code, // 保留原始代码
                                        };
                                        
                                        if !info_parts.contains(&vendor_name.to_string()) {
                                            info_parts.insert(0, vendor_name.to_string()); // 插入到开头作为品牌
                                            has_useful_info = true;
                                        }
                                    }
                                    
                                    // 尝试解析产品代码 (通常以DEV_开头或直接是产品代码)
                                    if sub_part.starts_with("DEV_") && sub_part.len() > 4 {
                                        let product_code = &sub_part[4..];
                                        if !info_parts.contains(&product_code.to_string()) && product_code.len() > 1 {
                                            info_parts.push(format!("Model: {}", product_code));
                                            has_useful_info = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 添加屏幕分辨率信息（如果可用）
                if let Some(width) = monitor.get("ScreenWidth") {
                    if let Some(height) = monitor.get("ScreenHeight") {
                        if let (wmi::Variant::UI4(width_val), wmi::Variant::UI4(height_val)) = (width, height) {
                            info_parts.push(format!("Resolution: {}x{}", width_val, height_val));
                            has_useful_info = true;
                        }
                    }
                }

                // 如果我们收集到了任何有用的信息（包括从PNPDeviceID中提取的），就添加到结果中
                if has_useful_info {
                    // 如果info_parts为空，至少包含原始名称
                    if info_parts.is_empty() {
                        if let Some(name) = monitor.get("Name") {
                            if let wmi::Variant::String(name_str) = name {
                                info_parts.push(name_str.clone());
                            }
                        }
                    }
                    monitor_info.push(info_parts.join(", "));
                } else {
                    // 如果没有提取到任何有用信息，至少记录一个基本名称
                    if let Some(name) = monitor.get("Name") {
                        if let wmi::Variant::String(name_str) = name {
                            if name_str != "Generic PnP Monitor" && name_str != "Default Monitor" {
                                monitor_info.push(name_str.clone());
                            }
                        }
                    } else if let Some(monitor_type) = monitor.get("MonitorType") {
                        if let wmi::Variant::String(type_str) = monitor_type {
                            if type_str != "默认监视器" && type_str != "通用即插即用监视器" {
                                monitor_info.push(type_str.clone());
                            }
                        }
                    }
                }
            }
        }

        // 如果通过Win32_DesktopMonitor仍没有获取到有用信息，尝试使用Win32_PnPEntity
        if monitor_info.is_empty() {
            let pnp_results: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT Description, DeviceID FROM Win32_PnPEntity WHERE Service='monitor'")
                .map_err(|e| format!("WMI query failed: {}", e))?;

            for device in pnp_results {
                if let Some(description) = device.get("Description") {
                    if let wmi::Variant::String(desc_str) = description {
                        // 检查是否是通用描述，如果是则跳过
                        if desc_str != "Generic PnP Monitor" && desc_str != "Default Monitor" {
                            let mut info_parts = vec![desc_str.clone()];
                            
                            // 从DeviceID中提取更多信息
                            if let Some(device_id) = device.get("DeviceID") {
                                if let wmi::Variant::String(device_id_str) = device_id {
                                    // 查找供应商ID (VEN_)
                                    if let Some(ven_pos) = device_id_str.find("VEN_") {
                                        let start = ven_pos + 4;
                                        let end = std::cmp::min(start + 4, device_id_str.len());
                                        let vendor_code = &device_id_str[start..end];
                                        
                                        // 尝试将供应商代码转换为品牌名
                                        let vendor_name = match vendor_code.to_uppercase().as_str() {
                                            "SAM" => "Samsung",
                                            "DEL" => "Dell", 
                                            "APP" => "Apple",
                                            "HWP" => "HP",
                                            "LEN" => "Lenovo",
                                            "ACR" => "Acer",
                                            "ASU" => "ASUS",
                                            "LGD" => "LG",
                                            "BNQ" => "BenQ",
                                            _ => vendor_code,
                                        };
                                        
                                        if !info_parts.contains(&vendor_name.to_string()) {
                                            info_parts.insert(0, vendor_name.to_string());
                                        }
                                    }
                                    
                                    // 查找产品ID (DEV_)
                                    if let Some(dev_pos) = device_id_str.find("DEV_") {
                                        let start = dev_pos + 4;
                                        let end = device_id_str[start..].find('#')
                                            .map(|pos| start + pos)
                                            .unwrap_or_else(|| std::cmp::min(start + 8, device_id_str.len()));
                                        let product_code = &device_id_str[start..end];
                                        
                                        if !product_code.is_empty() && !info_parts.contains(&format!("Model: {}", product_code)) {
                                            info_parts.push(format!("Model: {}", product_code));
                                        }
                                    }
                                }
                            }
                            
                            monitor_info.push(info_parts.join(" "));
                        }
                    }
                }
            }
        }

        // 如果仍然没有获取到具体信息，尝试使用更加详细的查询
        if monitor_info.is_empty() {
            // 最后的尝试：尝试使用Win32_VideoController来获取连接的显示器信息
            let video_results: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT Name, VideoModeDescription, CurrentHorizontalResolution, CurrentVerticalResolution FROM Win32_VideoController")
                .map_err(|e| format!("WMI query failed: {}", e))?;

            for video_controller in video_results {
                let mut info_parts = Vec::new();
                
                if let Some(name) = video_controller.get("Name") {
                    if let wmi::Variant::String(name_str) = name {
                        if !name_str.is_empty() && name_str != "Microsoft Basic Display Adapter" {
                            info_parts.push(name_str.clone());
                        }
                    }
                }
                
                if let Some(mode_desc) = video_controller.get("VideoModeDescription") {
                    if let wmi::Variant::String(desc_str) = mode_desc {
                        if !desc_str.is_empty() {
                            info_parts.push(desc_str.clone());
                        }
                    }
                }
                
                if let (Some(width), Some(height)) = (
                    video_controller.get("CurrentHorizontalResolution"),
                    video_controller.get("CurrentVerticalResolution")
                ) {
                    if let (wmi::Variant::UI4(width_val), wmi::Variant::UI4(height_val)) = (width, height) {
                        info_parts.push(format!("Resolution: {}x{}", width_val, height_val));
                    }
                }
                
                if !info_parts.is_empty() {
                    monitor_info.push(info_parts.join(", "));
                }
            }
        }

        // 如果仍然没有获取到具体信息，添加提示信息
        if monitor_info.is_empty() {
            monitor_info.push("未获取到显示器信息，检查功能代码的问题".to_string());
        }

        // 过滤掉通用显示器信息（如默认监视器、通用即插即用监视器等）
        // 但保留包含具体品牌或型号的显示器信息
        let real_monitor_info: Vec<String> = monitor_info.iter()
            .filter(|info| {
                // 完全通用的显示器信息才过滤
                let is_generic = info.as_str() == "默认监视器" || 
                                info.as_str() == "Default Monitor" || 
                                info.as_str() == "Generic PnP Monitor" ||
                                info.as_str() == "通用即插即用监视器";
                !is_generic
            })
            .cloned()
            .collect();

        // 添加显示器序号并调整格式为：显示器N：品牌-型号-分辨率+刷新率
        let formatted_monitor_info: Vec<String> = real_monitor_info.iter()
            .enumerate()
            .map(|(index, info)| {
                let monitor_number = index + 1;
                // 将现有的逗号分隔格式转换为破折号分隔
                let parts: Vec<&str> = info.split(", ").collect();
                let formatted_info = parts.join("-");
                format!("显示器{}：{}", monitor_number, formatted_info)
            })
            .collect();

        // 如果没有找到真实的显示器信息，添加提示
        if formatted_monitor_info.is_empty() {
            Ok(vec!["未检测到物理显示器".to_string()])
        } else {
            Ok(formatted_monitor_info)
        }
    }
}