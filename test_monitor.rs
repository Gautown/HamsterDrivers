use std::collections::HashMap;
use wmi::{WMIConnection, Variant};

fn main() {
    println!("开始测试显示器信息获取...");
    
    match WMIConnection::new(COMLibrary::new().unwrap().into()) {
        Ok(wmi_con) => {
            // 首先尝试从WmiMonitorID表获取真实的品牌型号信息
            if let Ok(monitor_id_results) = wmi_con.raw_query::<HashMap<String, Variant>>(
                "SELECT ManufacturerName, ProductCodeID, UserFriendlyName FROM WmiMonitorID"
            ) {
                println!("WmiMonitorID 查询结果数量: {}", monitor_id_results.len());
                
                for (i, monitor) in monitor_id_results.iter().enumerate() {
                    println!("显示器 {}: {:?}", i+1, monitor);
                    
                    let mut info_parts = Vec::new();

                    // 处理用户友好的显示器名称（如果有）
                    if let Some(friendly_name_raw) = monitor.get("UserFriendlyName") {
                        println!("  UserFriendlyName: {:?}", friendly_name_raw);
                    }
                    
                    // 处理制造商名称
                    if let Some(manufacturer_raw) = monitor.get("ManufacturerName") {
                        println!("  ManufacturerName: {:?}", manufacturer_raw);
                    }
                    
                    // 处理产品代码ID（型号）
                    if let Some(product_raw) = monitor.get("ProductCodeID") {
                        println!("  ProductCodeID: {:?}", product_raw);
                    }
                }
            } else {
                println!("无法查询 WmiMonitorID 表");
            }
            
            // 尝试从Win32_PnPEntity获取显示器信息
            if let Ok(pnp_results) = wmi_con.raw_query::<HashMap<String, Variant>>(
                "SELECT Name, Description, Manufacturer FROM Win32_PnPEntity WHERE Name LIKE '%Monitor%' OR Description LIKE '%Monitor%'"
            ) {
                println!("\nWin32_PnPEntity 查询结果数量: {}", pnp_results.len());
                
                for (i, pnp_entity) in pnp_results.iter().enumerate() {
                    println!("PnP设备 {}: {:?}", i+1, pnp_entity);
                }
            } else {
                println!("无法查询 Win32_PnPEntity 表");
            }
        }
        Err(e) => {
            eprintln!("建立WMI连接失败: {}", e);
        }
    }
}

use wmi::COMLibrary;