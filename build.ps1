# build.ps1 - 构建和打包脚本
$ErrorActionPreference = "Stop"

# 设置变量
$APP_NAME = "仓鼠驱动管家"
$VERSION = "1.0.0"
$OUTPUT_DIR = "dist"
$INSTALLER_DIR = "installer"

# 清理旧构建
Remove-Item -Path $OUTPUT_DIR -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $OUTPUT_DIR -Force

# 构建发布版本
Write-Host "Building release version..." -ForegroundColor Green
cargo build --release --target x86_64-pc-windows-msvc

# 复制文件
$RELEASE_DIR = "target\x86_64-pc-windows-msvc\release"
$BIN_FILES = @(
    "hamster-driver-manager.exe",
    "hamster-driver-manager.pdb"
)

foreach ($file in $BIN_FILES) {
    $source = Join-Path $RELEASE_DIR $file
    $dest = Join-Path $OUTPUT_DIR $file
    if (Test-Path $source) {
        Copy-Item $source $dest
    }
}

# 复制依赖项
$DEPS = @(
    "vcruntime140.dll",
    "vcruntime140_1.dll"
)

foreach ($dep in $DEPS) {
    Copy-Item $dep $OUTPUT_DIR -ErrorAction SilentlyContinue
}

# 创建NSIS安装脚本
$NSIS_SCRIPT = @"
!define APP_NAME "$APP_NAME"
!define APP_VERSION "$VERSION"
!define APP_PUBLISHER "Hamster Tools"
!define APP_EXE "hamster-driver-manager.exe"

Name "`$`{APP_NAME}"
OutFile "$INSTALLER_DIR\${APP_NAME}_${VERSION}_Setup.exe"
InstallDir "`$PROGRAMFILES64\`$`{APP_NAME}"
RequestExecutionLevel admin

!include MUI2.nsh

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"

Section "主程序"
    SetOutPath `$INSTDIR
    File /r "$OUTPUT_DIR\*"
    
    ; 创建开始菜单快捷方式
    CreateDirectory "`$SMPROGRAMS\`$`{APP_NAME}"
    CreateShortCut "`$SMPROGRAMS\`$`{APP_NAME}\`$`{APP_NAME}.lnk" "`$INSTDIR\`$`{APP_EXE}"
    CreateShortCut "`$DESKTOP\`$`{APP_NAME}.lnk" "`$INSTDIR\`$`{APP_EXE}"
    
    ; 写入卸载信息
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`$`{APP_NAME}" \
        "DisplayName" "`$`{APP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`$`{APP_NAME}" \
        "UninstallString" "`"`$INSTDIR\uninstall.exe`""
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`$`{APP_NAME}" \
        "DisplayIcon" "`$INSTDIR\`$`{APP_EXE}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`$`{APP_NAME}" \
        "Publisher" "`$`{APP_PUBLISHER}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`$`{APP_NAME}" \
        "DisplayVersion" "`$`{APP_VERSION}"
    
    ; 创建卸载程序
    WriteUninstaller "`$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "`$INSTDIR\uninstall.exe"
    RMDir /r "`$INSTDIR"
    
    Delete "`$SMPROGRAMS\`$`{APP_NAME}\`$`{APP_NAME}.lnk"
    RMDir "`$SMPROGRAMS\`$`{APP_NAME}"
    Delete "`$DESKTOP\`$`{APP_NAME}.lnk"
    
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\`$`{APP_NAME}"
SectionEnd
"@

New-Item -ItemType Directory -Path $INSTALLER_DIR -Force
$NSIS_SCRIPT | Out-File -FilePath "$INSTALLER_DIR\installer.nsi" -Encoding UTF8

Write-Host "Build completed!" -ForegroundColor Green
Write-Host "Output directory: $OUTPUT_DIR" -ForegroundColor Cyan