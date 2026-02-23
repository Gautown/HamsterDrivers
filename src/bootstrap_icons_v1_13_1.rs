// Bootstrap Icons v1.13.1 图标映射表
// 基于 Bootstrap Icons v1.13.1 的图标名称和对应的 Unicode 字符

use std::collections::HashMap;

/// Bootstrap Icons v1.13.1 图标映射表
pub fn get_bootstrap_icons_v1_13_1_map() -> HashMap<&'static str, char> {
    let mut icons = HashMap::new();
    
    // 基础形状图标
    icons.insert("square", '■');
    icons.insert("square-fill", '◼');
    icons.insert("r-square", '🅁');
    icons.insert("r-square-fill", '🅁');
    icons.insert("circle", '●');
    icons.insert("circle-fill", '●');
    icons.insert("triangle", '▲');
    icons.insert("triangle-fill", '▲');
    
    // 文档和文件相关图标
    icons.insert("file-earmark-text", '📄');
    icons.insert("file-text", '📄');
    icons.insert("file-earmark", '📄');
    icons.insert("file", '📄');
    
    // 信息和状态图标
    icons.insert("info-circle", 'ℹ');
    icons.insert("info-circle-fill", 'ℹ');
    icons.insert("exclamation-circle", '❗');
    icons.insert("exclamation-circle-fill", '❗');
    icons.insert("question-circle", '❓');
    icons.insert("question-circle-fill", '❓');
    icons.insert("check-circle", '✅');
    icons.insert("check-circle-fill", '✅');
    icons.insert("x-circle", '❌');
    icons.insert("x-circle-fill", '❌');
    
    // 数字和符号
    icons.insert("123", '🔢');
    icons.insert("hash", '#');
    icons.insert("gear", '⚙');
    icons.insert("gear-fill", '⚙');
    
    // 硬件相关图标
    icons.insert("pc", '💻');
    icons.insert("laptop", '💻');
    icons.insert("desktop", '🖥');
    icons.insert("motherboard", '🔌');
    icons.insert("cpu", '💻');
    icons.insert("circuit-board", '🔌');
    icons.insert("factory", '🏭');
    
    // 操作系统和品牌图标
    icons.insert("windows", '🪟');
    icons.insert("windows-logo", '🪟');
    
    // 网络和连接图标
    icons.insert("ethernet", '🔌');
    icons.insert("wifi", '📶');
    icons.insert("bluetooth", '📱');
    icons.insert("usb", '💾');
    
    // 显示和音频图标
    icons.insert("display", '🖥');
    icons.insert("sound-card", '🔊');
    
    // 存储图标
    icons.insert("hdd", '💾');
    icons.insert("ssd", '💾');
    
    // 显卡图标
    icons.insert("gpu-card", '🎮');
    
    icons
}

/// 获取 Bootstrap Icons v1.13.1 图标
pub fn get_bootstrap_icon_v1_13_1(name: &str) -> Option<(char, &'static str)> {
    let icons = get_bootstrap_icons_v1_13_1_map();
    
    if let Some(&icon_char) = icons.get(name) {
        Some((icon_char, "Bootstrap Icons v1.13.1"))
    } else {
        None
    }
}

/// 调试函数：检查 Bootstrap Icons v1.13.1 中可用的图标
pub fn debug_bootstrap_icons_v1_13_1() {
    println!("检查 Bootstrap Icons v1.13.1 中可用的图标:");
    println!("============================================");
    
    let icons = get_bootstrap_icons_v1_13_1_map();
    let test_icons = [
        "square", "square-fill", "circle", "circle-fill",
        "file-earmark-text", "info-circle", "123", "gear",
        "pc", "laptop", "motherboard", "cpu", "windows",
        "ethernet", "display", "hdd", "gpu-card"
    ];
    
    for icon_name in test_icons.iter() {
        if let Some(&icon_char) = icons.get(icon_name) {
            println!("✅ {}: 可用 (字符: {})", icon_name, icon_char);
        } else {
            println!("❌ {}: 不可用", icon_name);
        }
    }
}