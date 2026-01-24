// src/security/elevation.rs
use std::process::Command;
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcessToken,
    TokenElevation, GetTokenInformation,
};

pub fn require_admin() -> bool {
    unsafe {
        let mut token = 0;
        let process = GetCurrentProcess();
        
        if OpenProcessToken(process, 0x0008 /* TOKEN_QUERY */, &mut token) != 0 {
            let mut elevation: TokenElevation = std::mem::zeroed();
            let mut size = 0u32;
            
            if GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TokenElevation>() as u32,
                &mut size,
            ) != 0 {
                return elevation.TokenIsElevated != 0;
            }
        }
        
        false
    }
}

pub fn relaunch_as_admin() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let status = Command::new("powershell")
        .args(&[
            "-Command",
            "Start-Process",
            &format!("'{}'", exe_path.display()),
            "-Verb",
            "RunAs",
            "-ArgumentList",
            "'--elevated'"
        ])
        .status();
    
    match status {
        Ok(_) => std::process::exit(0),
        Err(e) => Err(format!("Failed to relaunch as admin: {}", e)),
    }
}