use std::collections::HashMap;
use wmi::{WMIConnection, Variant};
use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// EDID数据结构
#[derive(Debug, Clone)]
pub struct EdidInfo {
    pub manufacturer: String,
    pub product_code: String,
    pub serial_number: String,
    pub manufacture_week: u8,
    pub manufacture_year: u8,
    pub edid_version: u8,
    pub edid_revision: u8,
    pub screen_size_horizontal: Option<u8>,
    pub screen_size_vertical: Option<u8>,
    pub gamma: Option<f32>,
    pub supported_resolutions: Vec<String>,
}

/// 获取显示器EDID数据
pub fn get_edid_info(wmi_con: &WMIConnection) -> Result<Vec<EdidInfo>, String> {
    let mut edid_infos = Vec::new();
    
    // 尝试从WmiMonitorID获取EDID数据
    if let Ok(monitor_results) = wmi_con.raw_query::<HashMap<String, Variant>>(
        "SELECT ManufacturerName, ProductCodeID, SerialNumberID, WeekOfManufacture, YearOfManufacture, UserFriendlyName FROM WmiMonitorID"
    ) {
        for monitor in monitor_results {
            let mut edid_info = EdidInfo {
                manufacturer: String::new(),
                product_code: String::new(),
                serial_number: String::new(),
                manufacture_week: 0,
                manufacture_year: 0,
                edid_version: 1,
                edid_revision: 3,
                screen_size_horizontal: None,
                screen_size_vertical: None,
                gamma: None,
                supported_resolutions: Vec::new(),
            };
            
            // 解析制造商名称
            if let Some(manufacturer_raw) = monitor.get("ManufacturerName") {
                if let Variant::Array(bytes) = manufacturer_raw {
                    let manufacturer_bytes: Vec<u8> = bytes.iter()
                        .filter_map(|b| {
                            match b {
                                Variant::UI1(byte) if *byte != 0 => Some(*byte),
                                Variant::I8(byte) if *byte != 0 => Some(*byte as u8),
                                _ => None
                            }
                        })
                        .collect();
                    
                    if !manufacturer_bytes.is_empty() {
                        // 尝试多种编码格式
                        let manufacturer = decode_manufacturer_name(&manufacturer_bytes);
                        edid_info.manufacturer = manufacturer.trim().to_string();
                    }
                }
            }
            
            // 解析产品代码
            if let Some(product_raw) = monitor.get("ProductCodeID") {
                if let Variant::Array(bytes) = product_raw {
                    let product_bytes: Vec<u8> = bytes.iter()
                        .filter_map(|b| {
                            match b {
                                Variant::UI1(byte) if *byte != 0 => Some(*byte),
                                Variant::I8(byte) if *byte != 0 => Some(*byte as u8),
                                _ => None
                            }
                        })
                        .collect();
                    
                    if !product_bytes.is_empty() {
                        // 尝试多种编码格式
                        let product = decode_manufacturer_name(&product_bytes);
                        edid_info.product_code = product.trim().to_string();
                    }
                }
            }
            
            // 解析序列号
            if let Some(serial_raw) = monitor.get("SerialNumberID") {
                if let Variant::Array(bytes) = serial_raw {
                    let serial_bytes: Vec<u8> = bytes.iter()
                        .filter_map(|b| {
                            match b {
                                Variant::UI1(byte) if *byte != 0 => Some(*byte),
                                Variant::I8(byte) if *byte != 0 => Some(*byte as u8),
                                _ => None
                            }
                        })
                        .collect();
                    
                    if !serial_bytes.is_empty() {
                        let serial = String::from_utf8(serial_bytes.clone())
                            .unwrap_or_else(|_| {
                                serial_bytes.iter()
                                    .map(|&b| b as char)
                                    .collect()
                            });
                        edid_info.serial_number = serial.trim().to_string();
                    }
                }
            }
            
            // 解析制造日期
            if let Some(week_raw) = monitor.get("WeekOfManufacture") {
                if let Variant::UI2(week) = week_raw {
                    edid_info.manufacture_week = (*week as u8).min(52);
                }
            }
            
            if let Some(year_raw) = monitor.get("YearOfManufacture") {
                if let Variant::UI2(year) = year_raw {
                    edid_info.manufacture_year = (*year as u8).min(99);
                }
            }
            
            // 过滤掉无效的数据
            if !edid_info.manufacturer.is_empty() && 
               !edid_info.manufacturer.contains("0000") &&
               edid_info.manufacturer.len() > 2 {
                edid_infos.push(edid_info);
            }
        }
    }
    
    // 如果从WmiMonitorID没有获取到足够信息，尝试从其他来源获取
    if edid_infos.is_empty() {
        // 尝试从Win32_DesktopMonitor获取基本信息
        if let Ok(desktop_results) = wmi_con.raw_query::<HashMap<String, Variant>>(
            "SELECT MonitorManufacturer, Name, ScreenWidth, ScreenHeight FROM Win32_DesktopMonitor"
        ) {
            for desktop_monitor in desktop_results {
                let mut edid_info = EdidInfo {
                    manufacturer: String::new(),
                    product_code: String::new(),
                    serial_number: String::new(),
                    manufacture_week: 0,
                    manufacture_year: 0,
                    edid_version: 1,
                    edid_revision: 3,
                    screen_size_horizontal: None,
                    screen_size_vertical: None,
                    gamma: None,
                    supported_resolutions: Vec::new(),
                };
                
                // 获取制造商
                if let Some(manufacturer) = desktop_monitor.get("MonitorManufacturer") {
                    if let Variant::String(manufacturer_str) = manufacturer {
                        edid_info.manufacturer = manufacturer_str.trim().to_string();
                    }
                }
                
                // 获取名称作为产品代码
                if let Some(name) = desktop_monitor.get("Name") {
                    if let Variant::String(name_str) = name {
                        edid_info.product_code = name_str.trim().to_string();
                    }
                }
                
                // 获取屏幕尺寸
                if let Some(width) = desktop_monitor.get("ScreenWidth") {
                    if let Variant::UI4(width_val) = width {
                        edid_info.screen_size_horizontal = Some(*width_val as u8);
                    }
                }
                
                if let Some(height) = desktop_monitor.get("ScreenHeight") {
                    if let Variant::UI4(height_val) = height {
                        edid_info.screen_size_vertical = Some(*height_val as u8);
                    }
                }
                
                if !edid_info.manufacturer.is_empty() && 
                   !edid_info.manufacturer.contains("Generic") &&
                   !edid_info.manufacturer.contains("通用") {
                    edid_infos.push(edid_info);
                }
            }
        }
    }
    
    Ok(edid_infos)
}

/// 将EDID信息格式化为可读字符串
pub fn format_edid_info(edid_info: &EdidInfo) -> String {
    let mut parts = Vec::new();
    
    // 制造商
    if !edid_info.manufacturer.is_empty() {
        parts.push(edid_info.manufacturer.clone());
    } else {
        parts.push("未知制造商".to_string());
    }
    
    // 型号
    if !edid_info.product_code.is_empty() {
        parts.push(edid_info.product_code.clone());
    } else {
        parts.push("未知型号".to_string());
    }
    
    // 屏幕尺寸（英寸）
    if let (Some(width), Some(height)) = (edid_info.screen_size_horizontal, edid_info.screen_size_vertical) {
        if width > 0 && height > 0 {
            // 转换为英寸（1英寸=2.54厘米）
            let diagonal_inches = ((width as f32).powi(2) + (height as f32).powi(2)).sqrt() / 2.54;
            parts.push(format!("{:.0}英寸", diagonal_inches.round()));
        } else {
            parts.push("未知尺寸".to_string());
        }
    } else {
        parts.push("未知尺寸".to_string());
    }
    
    if parts.is_empty() {
        "未知显示器".to_string()
    } else {
        parts.join("-")
    }
}

/// 解码制造商名称，处理多种字符编码
fn decode_manufacturer_name(bytes: &[u8]) -> String {
    // 首先尝试UTF-8
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        let trimmed = s.trim_matches('\0').trim();
        if !trimmed.is_empty() && !trimmed.chars().all(|c| c == '\0' || c.is_control()) {
            return trimmed.to_string();
        }
    }
    
    // 尝试ASCII（直接映射）
    let ascii_str: String = bytes.iter()
        .filter(|&&b| b != 0)
        .map(|&b| {
            if b >= 32 && b <= 126 {
                b as char
            } else {
                '?' // 替换不可打印字符
            }
        })
        .collect();
    
    if !ascii_str.trim().is_empty() {
        return ascii_str.trim().to_string();
    }
    
    // 如果都失败，返回未知
    "未知制造商".to_string()
}

/// 使用PowerShell直接获取显示器信息
pub fn get_direct_monitor_info() -> Result<Vec<String>, String> {
    let mut monitor_info = Vec::new();
    
    // 使用PowerShell命令获取显示器信息，隐藏窗口
    let output = Command::new("powershell")
        .args([
            "-WindowStyle", "Hidden",
            "-Command",
            "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; 
            Get-WmiObject -Namespace root\\wmi -Class WmiMonitorID | ForEach-Object { 
                # 使用UTF-8编码处理制造商名称
                $manufacturerBytes = $_.ManufacturerName | Where-Object { $_ -ne 0 }
                $manufacturer = [System.Text.Encoding]::UTF8.GetString($manufacturerBytes).Trim()
                if ([string]::IsNullOrEmpty($manufacturer)) {
                    $manufacturer = [System.Text.Encoding]::ASCII.GetString($manufacturerBytes).Trim()
                }
                
                # 使用UTF-8编码处理产品代码
                $productBytes = $_.ProductCodeID | Where-Object { $_ -ne 0 }
                $product = [System.Text.Encoding]::UTF8.GetString($productBytes).Trim()
                if ([string]::IsNullOrEmpty($product)) {
                    $product = [System.Text.Encoding]::ASCII.GetString($productBytes).Trim()
                }
                
                if ($manufacturer -and $product -and $manufacturer -ne '0000' -and $manufacturer.Length -gt 2) {
                    Write-Output (\"$manufacturer-$product\")
                }
            }; 
            Get-WmiObject -Class Win32_DesktopMonitor | Where-Object { $_.MonitorManufacturer -ne $null -and $_.MonitorManufacturer -ne 'None' -and $_.MonitorManufacturer -ne '(标准监视器类型)' } | ForEach-Object { 
                Write-Output (\"$($_.MonitorManufacturer)-$($_.Name)\")
            };
            # 尝试获取屏幕尺寸信息
            Get-WmiObject -Namespace root\\wmi -Class WmiMonitorBasicDisplayParams | ForEach-Object { 
                if ($_.MaxHorizontalImageSize -and $_.MaxVerticalImageSize) {
                    $width = $_.MaxHorizontalImageSize
                    $height = $_.MaxVerticalImageSize
                    $diagonal = [Math]::Sqrt($width * $width + $height * $height) / 2.54
                    Write-Output (\"尺寸:$([Math]::Round($diagonal))英寸\")
                }
            }"
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            
            // 分离显示器信息和尺寸信息
            let mut monitor_lines = Vec::new();
            let mut size_lines = Vec::new();
            
            for line in &lines {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if trimmed.starts_with("尺寸:") {
                        size_lines.push(trimmed);
                    } else {
                        monitor_lines.push(trimmed);
                    }
                }
            }
            
            for (i, monitor_line) in monitor_lines.iter().enumerate() {
                // 获取当前分辨率信息
                let resolution_output = Command::new("powershell")
                    .args([
                        "-WindowStyle", "Hidden",
                        "-Command",
                        "Get-WmiObject -Class Win32_VideoController | Where-Object { $_.CurrentHorizontalResolution -ne $null -and $_.CurrentHorizontalResolution -gt 0 } | ForEach-Object { 
                            $width = $_.CurrentHorizontalResolution
                            $height = $_.CurrentVerticalResolution
                            $refresh = if ($_.CurrentRefreshRate) { $_.CurrentRefreshRate } else { '?' }
                            Write-Output (\"${width}x${height}@${refresh}Hz\")
                        }"
                    ])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output();
                
                let resolution = match resolution_output {
                    Ok(res_output) if res_output.status.success() => {
                        let stdout = String::from_utf8_lossy(&res_output.stdout);
                        let lines: Vec<&str> = stdout.lines().collect();
                        if !lines.is_empty() {
                            lines[0].trim().to_string()
                        } else {
                            "?x?@?Hz".to_string()
                        }
                    }
                    _ => "?x?@?Hz".to_string()
                };
                
                // 获取对应的屏幕尺寸
                let screen_size = if i < size_lines.len() {
                    let size_str = size_lines[i];
                    if size_str.starts_with("尺寸:") {
                        size_str.replace("尺寸:", "")
                    } else {
                        "未知尺寸".to_string()
                    }
                } else {
                    "未知尺寸".to_string()
                };
                
                // 格式化为：制造商-型号-屏幕尺寸-分辨率@刷新率
                let parts: Vec<&str> = monitor_line.split('-').collect();
                let formatted_info = if parts.len() >= 2 {
                    format!("{}-{}-{}", monitor_line, screen_size, resolution)
                } else {
                    format!("{}-未知型号-{}-{}", monitor_line, screen_size, resolution)
                };
                monitor_info.push(formatted_info);
            }
            
            if monitor_info.is_empty() {
                // 如果PowerShell没有获取到信息，尝试使用系统信息
                let sys_output = Command::new("powershell")
                    .args([
                        "-WindowStyle", "Hidden",
                        "-Command",
                        "Get-WmiObject -Class Win32_VideoController | Where-Object { $_.CurrentHorizontalResolution -ne $null -and $_.CurrentHorizontalResolution -gt 0 } | ForEach-Object { 
                            $width = $_.CurrentHorizontalResolution
                            $height = $_.CurrentVerticalResolution
                            $refresh = if ($_.CurrentRefreshRate) { $_.CurrentRefreshRate } else { '?' }
                            Write-Output (\"${width}x${height}@${refresh}Hz\")
                        }"
                    ])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output();
                
                match sys_output {
                    Ok(sys_output) if sys_output.status.success() => {
                        let stdout = String::from_utf8_lossy(&sys_output.stdout);
                        let lines: Vec<&str> = stdout.lines().collect();
                        
                        for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // 格式化为：未知制造商-未知型号-未知尺寸-分辨率@刷新率
                        monitor_info.push(format!("未知制造商-未知型号-未知尺寸-{}", trimmed));
                    }
                }
                    }
                    _ => {}
                }
            }
            
            if monitor_info.is_empty() {
                Ok(vec!["未检测到显示器信息".to_string()])
            } else {
                Ok(monitor_info)
            }
        }
        Err(e) => Err(format!("执行PowerShell命令失败: {}", e)),
        _ => Err("PowerShell命令执行失败".to_string())
    }
}

/// 获取完整的显示器信息（包括EDID和当前分辨率）
pub fn get_complete_monitor_info(wmi_con: &WMIConnection) -> Result<Vec<String>, String> {
    // 首先尝试使用Windows API直接获取显示器信息
    match get_direct_monitor_info() {
        Ok(info) if !info.is_empty() && !info[0].contains("未检测到物理显示器") => {
            // 如果Windows API成功获取到信息，直接返回
            return Ok(info);
        }
        _ => {
            // Windows API失败，回退到WMI查询
        }
    }
    
    let mut monitor_info = Vec::new();
    
    // 获取EDID信息
    let edid_infos = get_edid_info(wmi_con)?;
    
    // 获取当前分辨率和刷新率
    let mut resolution_info = Vec::new();
    if let Ok(video_results) = wmi_con.raw_query::<HashMap<String, Variant>>(
        "SELECT CurrentHorizontalResolution, CurrentVerticalResolution, CurrentRefreshRate FROM Win32_VideoController"
    ) {
        for video_controller in video_results {
            let mut resolution_str = String::new();
            
            if let (Some(width), Some(height)) = (
                video_controller.get("CurrentHorizontalResolution"),
                video_controller.get("CurrentVerticalResolution")
            ) {
                if let (Variant::UI4(width_val), Variant::UI4(height_val)) = (width, height) {
                    resolution_str = format!("{}x{}", width_val, height_val);
                }
            }
            
            let mut refresh_rate_str = String::new();
            if let Some(refresh_rate) = video_controller.get("CurrentRefreshRate") {
                if let Variant::UI4(refresh_val) = refresh_rate {
                    refresh_rate_str = format!("{}Hz", refresh_val);
                }
            }
            
            if !resolution_str.is_empty() {
                if !refresh_rate_str.is_empty() {
                    resolution_info.push(format!("{}@{}", resolution_str, refresh_rate_str));
                } else {
                    resolution_info.push(format!("{}@?Hz", resolution_str));
                }
            }
        }
    }
    
    // 组合EDID信息和分辨率信息
    for (i, edid_info) in edid_infos.iter().enumerate() {
        let edid_str = format_edid_info(edid_info);
        let res_info = if i < resolution_info.len() {
            &resolution_info[i]
        } else {
            "?x?@?Hz"
        };
        
        // 格式化为：制造商-型号-屏幕尺寸-分辨率@刷新率
        monitor_info.push(format!("{}-{}", edid_str, res_info));
    }
    
    // 如果没有获取到EDID信息，但获取到了分辨率信息
    if monitor_info.is_empty() && !resolution_info.is_empty() {
        for (i, res_info) in resolution_info.iter().enumerate() {
            monitor_info.push(format!("显示器{}：{}", i + 1, res_info));
        }
    }
    
    // 如果仍然没有信息，返回默认提示
    if monitor_info.is_empty() {
        Ok(vec!["未检测到显示器信息".to_string()])
    } else {
        Ok(monitor_info)
    }
}