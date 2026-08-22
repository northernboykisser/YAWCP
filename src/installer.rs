use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_appdata_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("yawcp")
    } else {
        PathBuf::from(".").join("yawcp")
    }
}

pub fn install_to_appdata() -> Result<PathBuf, String> {
    let appdata_dir = get_appdata_dir();
    fs::create_dir_all(&appdata_dir).map_err(|e| e.to_string())?;

    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;

    let target_yawcp = appdata_dir.join("yawcp.exe");
    let target_ping = appdata_dir.join("ping.exe");

    if current_exe != target_yawcp {
        let _ = fs::copy(&current_exe, &target_yawcp);
    }
    if current_exe != target_ping {
        let _ = fs::copy(&current_exe, &target_ping);
    }

    let rus_path = appdata_dir.join("rus.json");
    if !rus_path.exists() {
        let _ = fs::write(&rus_path, include_str!("../rus.json"));
    }

    let eng_path = appdata_dir.join("eng.json");
    if !eng_path.exists() {
        let _ = fs::write(&eng_path, include_str!("../eng.json"));
    }

    add_to_user_path(&appdata_dir);
    setup_powershell_profile(&appdata_dir);

    Ok(appdata_dir)
}

fn add_to_user_path(dir: &Path) {
    let dir_str = dir.to_string_lossy();
    let ps_cmd = format!(
        "$dir = '{}'; $p = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($p -notlike \"*$dir*\") {{ [Environment]::SetEnvironmentVariable('Path', \"$dir;$p\", 'User') }}",
        dir_str
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .output();
}

fn setup_powershell_profile(dir: &Path) {
    let yawcp_exe = dir.join("yawcp.exe");
    let yawcp_str = yawcp_exe.to_string_lossy();
    let function_def = format!("\nfunction global:ping {{ & \"{}\" @args }}\n", yawcp_str);

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let doc_dir = PathBuf::from(userprofile).join("Documents");

        let profiles = [
            doc_dir.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"),
            doc_dir.join("PowerShell").join("Microsoft.PowerShell_profile.ps1"),
        ];

        for profile in &profiles {
            if let Some(parent) = profile.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let existing = fs::read_to_string(profile).unwrap_or_default();
            if !existing.contains("function global:ping") {
                let mut new_content = existing;
                new_content.push_str(&function_def);
                let _ = fs::write(profile, new_content);
            }
        }
    }
}

pub fn ensure_installed() {
    let appdata_dir = get_appdata_dir();
    let target_yawcp = appdata_dir.join("yawcp.exe");
    let target_ping = appdata_dir.join("ping.exe");

    if !target_yawcp.exists() || !target_ping.exists() {
        let _ = install_to_appdata();
    }
}
