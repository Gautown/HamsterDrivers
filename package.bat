@echo off
echo 正在构建仓鼠驱动管家...

:: 清理旧构建
if exist "dist" rmdir /s /q "dist"
if exist "installer" rmdir /s /q "installer"

:: 创建输出目录
mkdir dist
mkdir installer

:: 构建发布版本
echo 构建发布版本...
cargo build --release

:: 复制可执行文件到dist目录
copy "target\release\hamster-driver-manager.exe" "dist\"

:: 复制资源文件
if exist "assets" xcopy "assets" "dist\assets" /E /I /Y

:: 创建简单的安装脚本
echo 创建安装脚本...
(
echo @echo off
echo echo 正在安装仓鼠驱动管家...
echo.
echo if not exist "%%PROGRAMFILES%%\仓鼠驱动管家" mkdir "%%PROGRAMFILES%%\仓鼠驱动管家"
echo xcopy "%%~dp0*" "%%PROGRAMFILES%%\仓鼠驱动管家\" /E /I /Y
echo echo 创建桌面快捷方式...
echo powershell -Command "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%%USERPROFILE%%\Desktop\\仓鼠驱动管家.lnk'); $Shortcut.TargetPath = '%%PROGRAMFILES%%\\仓鼠驱动管家\\hamster-driver-manager.exe'; $Shortcut.Save()"
echo echo 安装完成！
echo pause
) > "installer\install.bat"

echo 构建完成！
echo 可执行文件位置: dist\hamster-driver-manager.exe
echo 安装脚本位置: installer\install.bat
pause