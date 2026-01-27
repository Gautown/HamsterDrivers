# 仓鼠驱动管家打包脚本
Write-Host "正在构建仓鼠驱动管家..." -ForegroundColor Green

# 清理旧构建
if (Test-Path "dist") { Remove-Item -Path "dist" -Recurse -Force }
if (Test-Path "installer") { Remove-Item -Path "installer" -Recurse -Force }

# 创建输出目录
New-Item -ItemType Directory -Path "dist" -Force
New-Item -ItemType Directory -Path "installer" -Force

# 构建发布版本
Write-Host "构建发布版本..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "构建失败，请检查错误信息" -ForegroundColor Red
    exit 1
}

# 复制可执行文件到dist目录
Copy-Item "target\release\hamster-driver-manager.exe" "dist\" -Force

# 复制资源文件
if (Test-Path "assets") {
    Copy-Item "assets" "dist\assets" -Recurse -Force
}

# 创建安装脚本
Write-Host "创建安装脚本..." -ForegroundColor Yellow

$installScript = @"
# 仓鼠驱动管家安装脚本
Write-Host \"正在安装仓鼠驱动管家...\" -ForegroundColor Green

# 创建安装目录
`$installDir = \"`$env:PROGRAMFILES\\仓鼠驱动管家\"
if (-not (Test-Path `$installDir)) {
    New-Item -ItemType Directory -Path `$installDir -Force
}

# 复制文件
Copy-Item \"*\" `$installDir -Recurse -Force

# 创建桌面快捷方式
Write-Host \"创建桌面快捷方式...\" -ForegroundColor Yellow
`$WshShell = New-Object -comObject WScript.Shell
`$Shortcut = `$WshShell.CreateShortcut(\"`$env:USERPROFILE\\Desktop\\仓鼠驱动管家.lnk\")
`$Shortcut.TargetPath = \"`$installDir\\hamster-driver-manager.exe\"
`$Shortcut.WorkingDirectory = `$installDir
`$Shortcut.Save()

Write-Host \"安装完成！\" -ForegroundColor Green
Write-Host \"程序已安装到: `$installDir\" -ForegroundColor Cyan
Write-Host \"桌面快捷方式已创建\" -ForegroundColor Cyan
Read-Host \"按回车键退出\"
"@

Set-Content -Path "installer\install.ps1" -Value $installScript

# 创建卸载脚本
$uninstallScript = @"
# 仓鼠驱动管家卸载脚本
Write-Host \"正在卸载仓鼠驱动管家...\" -ForegroundColor Yellow

`$installDir = \"`$env:PROGRAMFILES\\仓鼠驱动管家\"

# 删除安装目录
if (Test-Path `$installDir) {
    Remove-Item -Path `$installDir -Recurse -Force
}

# 删除桌面快捷方式
`$desktopShortcut = \"`$env:USERPROFILE\\Desktop\\仓鼠驱动管家.lnk\"
if (Test-Path `$desktopShortcut) {
    Remove-Item -Path `$desktopShortcut -Force
}

Write-Host \"卸载完成！\" -ForegroundColor Green
Read-Host \"按回车键退出\"
"@

Set-Content -Path "installer\uninstall.ps1" -Value $uninstallScript

Write-Host "构建完成！" -ForegroundColor Green
Write-Host "可执行文件位置: dist\hamster-driver-manager.exe" -ForegroundColor Cyan
Write-Host "安装脚本位置: installer\install.ps1" -ForegroundColor Cyan
Write-Host "卸载脚本位置: installer\uninstall.ps1" -ForegroundColor Cyan