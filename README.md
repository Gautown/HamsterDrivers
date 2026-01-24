# 仓鼠驱动管家 (Hamster Driver Manager)

一个现代化的 Windows 驱动管理工具，帮助用户轻松管理和维护系统驱动程序。

## 功能特性

- 驱动扫描和检测
- 驱动备份与恢复
- 驱动签名验证
- 可视化界面管理

## 技术栈

- Rust 语言开发
- egui 图形界面框架
- Windows API 集成

## 安装要求

- Rust 1.93.0 或更高版本
- Windows 操作系统
- MinGW-w64 工具链 (用于编译)

## 构建说明

```bash
# 克隆仓库
git clone <repository-url>

# 进入项目目录
cd HamsterDrivers

# 构建项目
cargo build --release

# 运行项目
cargo run
```

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](./LICENSE) 文件了解详情。