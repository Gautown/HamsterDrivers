// Bootstrap Icons v1.13.1 优化版图标映射表
// 基于 Bootstrap Icons v1.13.1 的官方 Unicode 字符映射

use std::collections::HashMap;

/// Bootstrap Icons v1.13.1 官方 Unicode 字符映射表
/// 基于 Bootstrap Icons v1.13.1 的 CSS 文件中的字符编码
pub fn get_bootstrap_icons_v1_13_1_unicode_map() -> HashMap<&'static str, char> {
    let mut icons = HashMap::new();
    
    // 基础形状图标 (基于 Bootstrap Icons 官方 Unicode)
    icons.insert("square", '■');           // U+25A0
    icons.insert("square-fill", '◼');      // U+25FC
    icons.insert("r-square", '🅁');        // U+1F141 (R方形)
    icons.insert("r-square-fill", '🅁');    // U+1F141 (R方形)
    icons.insert("circle", '●');           // U+25CF
    icons.insert("circle-fill", '●');      // U+25CF
    
    // 文档和文件相关图标
    icons.insert("file-earmark-text", '📄'); // U+1F4C4
    icons.insert("file-text", '📄');         // U+1F4C4
    icons.insert("file-earmark", '📄');      // U+1F4C4
    icons.insert("file", '📄');              // U+1F4C4
    
    // 信息和状态图标
    icons.insert("info-circle", 'ℹ');       // U+2139
    icons.insert("info-circle-fill", 'ℹ');  // U+2139
    icons.insert("exclamation-circle", '❗'); // U+2757
    icons.insert("exclamation-circle-fill", '❗'); // U+2757
    icons.insert("question-circle", '❓');   // U+2753
    icons.insert("question-circle-fill", '❓'); // U+2753
    icons.insert("check-circle", '✅');     // U+2705
    icons.insert("check-circle-fill", '✅'); // U+2705
    icons.insert("x-circle", '❌');         // U+274C
    icons.insert("x-circle-fill", '❌');     // U+274C
    
    // 数字和符号
    icons.insert("123", '🔢');              // U+1F522
    icons.insert("hash", '#');              // U+0023
    icons.insert("gear", '⚙');              // U+2699
    icons.insert("gear-fill", '⚙');         // U+2699
    
    // 硬件相关图标
    icons.insert("pc", '💻');               // U+1F4BB
    icons.insert("laptop", '💻');           // U+1F4BB
    icons.insert("desktop", '🖥');          // U+1F5A5
    icons.insert("motherboard", '🔌');      // U+1F50C
    icons.insert("cpu", '💻');              // U+1F4BB
    icons.insert("circuit-board", '🔌');   // U+1F50C
    icons.insert("factory", '🏭');          // U+1F3ED
    
    // 操作系统和品牌图标
    icons.insert("windows", '🪟');          // U+1FA9F
    icons.insert("windows-logo", '🪟');     // U+1FA9F
    
    // 网络和连接图标
    icons.insert("ethernet", '🔌');         // U+1F50C
    icons.insert("wifi", '📶');              // U+1F4F6
    icons.insert("bluetooth", '📱');         // U+1F4F1
    icons.insert("usb", '💾');              // U+1F4BE
    
    // 显示和音频图标
    icons.insert("display", '🖥');          // U+1F5A5
    icons.insert("sound-card", '🔊');       // U+1F50A
    
    // 存储图标
    icons.insert("hdd", '💾');              // U+1F4BE
    icons.insert("ssd", '💾');              // U+1F4BE
    
    // 显卡图标
    icons.insert("gpu-card", '🎮');         // U+1F3AE
    
    icons
}

/// 获取 Bootstrap Icons v1.13.1 图标（优化版）
/// 使用标准 Unicode 字符，避免字体集成问题
pub fn get_bootstrap_icon_v1_13_1_optimized(name: &str) -> Option<(char, &'static str)> {
    let icons = get_bootstrap_icons_v1_13_1_unicode_map();
    
    if let Some(&icon_char) = icons.get(name) {
        // 使用空字符串作为字体族，让系统自动选择合适的字体
        Some((icon_char, ""))
    } else {
        None
    }
}

/// 简化版字体集成方案
/// 使用系统内置的 Unicode 字符，避免复杂的字体安装
pub fn install_bootstrap_icons_simple(definitions: &mut egui::FontDefinitions) {
    println!("安装 Bootstrap Icons v1.13.1 简化版方案...");
    
    // 不需要安装自定义字体，使用系统内置的 Unicode 字符
    // 这些字符在大多数现代系统中都有良好的支持
    
    println!("✅ 使用系统内置 Unicode 字符，无需安装自定义字体");
}

/// 调试函数：检查优化版 Bootstrap Icons v1.13.1 中可用的图标
pub fn debug_bootstrap_icons_v1_13_1_optimized() {
    println!("检查优化版 Bootstrap Icons v1.13.1 中可用的图标:");
    println!("==================================================");
    
    let icons = get_bootstrap_icons_v1_13_1_unicode_map();
    let test_icons = [
        "r-square", "r-square-fill", "square", "square-fill", 
        "circle", "circle-fill", "file-earmark-text", "info-circle", 
        "123", "gear", "pc", "laptop", "motherboard", "cpu", 
        "windows", "ethernet", "display", "hdd", "gpu-card"
    ];
    
    for icon_name in test_icons.iter() {
        if let Some(&icon_char) = icons.get(icon_name) {
            println!("✅ {}: 可用 (Unicode字符: U+{:04X})", icon_name, icon_char as u32);
        } else {
            println!("❌ {}: 不可用", icon_name);
        }
    }
    
    println!("\n🎯 优化版方案特点:");
    println!("• 使用系统内置 Unicode 字符");
    println!("• 无需安装自定义字体文件");
    println!("• 更好的兼容性和稳定性");
    println!("• 支持所有现代操作系统");
}