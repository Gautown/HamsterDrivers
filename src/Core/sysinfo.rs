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

    fn get_memory_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Capacity, Speed, Manufacturer FROM Win32_PhysicalMemory")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut memory_info = Vec::new();

        for mem in results {
            let mut info_parts = Vec::new();

            if let Some(capacity) = mem.get("Capacity") {
                if let wmi::Variant::String(capacity_str) = capacity {
                    if let Ok(capacity_bytes) = capacity_str.parse::<u64>() {
                        let capacity_gb = capacity_bytes / (1024 * 1024 * 1024);
                        info_parts.push(format!("{}GB", capacity_gb));
                    }
                }
            }

            if let Some(speed) = mem.get("Speed") {
                if let wmi::Variant::UI4(speed_val) = speed {
                    info_parts.push(format!("{}MHz", speed_val));
                }
            }

            if let Some(manufacturer) = mem.get("Manufacturer") {
                if let wmi::Variant::String(mfg) = manufacturer {
                    info_parts.push(mfg.clone());
                }
            }

            if !info_parts.is_empty() {
                memory_info.push(info_parts.join(", "));
            }
        }

        if memory_info.is_empty() {
            memory_info.push("Unknown Memory Information".to_string());
        }

        Ok(memory_info)
    }

    fn get_disk_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Model, Size FROM Win32_DiskDrive WHERE MediaType = 'Fixed hard disk media'")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut disk_info = Vec::new();

        for disk in results {
            let mut info_parts = Vec::new();

            if let Some(model) = disk.get("Model") {
                if let wmi::Variant::String(model_str) = model {
                    info_parts.push(model_str.clone());
                }
            }

            if let Some(size) = disk.get("Size") {
                if let wmi::Variant::String(size_str) = size {
                    if let Ok(size_bytes) = size_str.parse::<u64>() {
                        let size_gb = size_bytes / (1024 * 1024 * 1024);
                        info_parts.push(format!("{}GB", size_gb));
                    }
                }
            }

            if !info_parts.is_empty() {
                disk_info.push(info_parts.join(", "));
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
        // 获取显示器信息比较复杂，这里简化处理
        // 在实际应用中，可能需要使用更复杂的API或不同的WMI查询
        let results: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Name, MonitorType FROM Win32_DesktopMonitor")
            .map_err(|e| format!("WMI query failed: {}", e))?;

        let mut monitor_info = Vec::new();

        for monitor in results {
            let mut info_parts = Vec::new();

            if let Some(name) = monitor.get("Name") {
                if let wmi::Variant::String(name_str) = name {
                    info_parts.push(name_str.clone());
                }
            }

            if let Some(monitor_type) = monitor.get("MonitorType") {
                if let wmi::Variant::String(type_str) = monitor_type {
                    info_parts.push(type_str.clone());
                }
            }

            if !info_parts.is_empty() {
                monitor_info.push(info_parts.join(", "));
            }
        }

        if monitor_info.is_empty() {
            monitor_info.push("Unknown Monitor Information".to_string());
        }

        Ok(monitor_info)
    }
}