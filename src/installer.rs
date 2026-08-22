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
    setup_cmd_autorun(&appdata_dir);
    setup_powershell_profiles(&appdata_dir);
    setup_bash_profiles(&appdata_dir);
    setup_app_paths(&appdata_dir);

    Ok(appdata_dir)
}

pub fn uninstall() -> Result<(), String> {
    let appdata_dir = get_appdata_dir();

    remove_from_user_path(&appdata_dir);
    remove_cmd_autorun();
    remove_powershell_profiles();
    remove_bash_profiles();
    remove_app_paths();

    if appdata_dir.exists() {
        let _ = fs::remove_dir_all(&appdata_dir);
    }

    Ok(())
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

fn remove_from_user_path(dir: &Path) {
    let dir_str = dir.to_string_lossy();
    let ps_cmd = format!(
        "$dir = '{}'; $p = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($p) {{ $newP = ($p.Split(';') | Where-Object {{ $_ -ne $dir -and $_ -ne '' }}) -join ';'; [Environment]::SetEnvironmentVariable('Path', $newP, 'User') }}",
        dir_str
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .output();
}

fn setup_cmd_autorun(dir: &Path) {
    let yawcp_exe = dir.join("yawcp.exe");
    let autorun_cmd = dir.join("autorun.cmd");
    let content = format!(
        "@doskey ping=\"{}\" $*\n@doskey yawcp=\"{}\" $*\n",
        yawcp_exe.to_string_lossy(),
        yawcp_exe.to_string_lossy()
    );
    let _ = fs::write(&autorun_cmd, content);

    let autorun_path = autorun_cmd.to_string_lossy();
    let _ = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Command Processor",
            "/v",
            "AutoRun",
            "/t",
            "REG_SZ",
            "/d",
            &autorun_path,
            "/f",
        ])
        .output();
}

fn remove_cmd_autorun() {
    let _ = Command::new("reg")
        .args([
            "delete",
            "HKCU\\Software\\Microsoft\\Command Processor",
            "/v",
            "AutoRun",
            "/f",
        ])
        .output();
}

fn setup_powershell_profiles(dir: &Path) {
    let yawcp_exe = dir.join("yawcp.exe");
    let yawcp_str = yawcp_exe.to_string_lossy();
    let function_def = format!(
        "\nfunction global:ping {{ & \"{}\" @args }}\nfunction global:yawcp {{ & \"{}\" @args }}\n",
        yawcp_str, yawcp_str
    );

    let _ = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned -Force",
        ])
        .output();

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let doc_dir = PathBuf::from(userprofile).join("Documents");

        let profiles = [
            doc_dir.join("WindowsPowerShell").join("profile.ps1"),
            doc_dir.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"),
            doc_dir.join("PowerShell").join("profile.ps1"),
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

fn remove_powershell_profiles() {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let doc_dir = PathBuf::from(userprofile).join("Documents");

        let profiles = [
            doc_dir.join("WindowsPowerShell").join("profile.ps1"),
            doc_dir.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"),
            doc_dir.join("PowerShell").join("profile.ps1"),
            doc_dir.join("PowerShell").join("Microsoft.PowerShell_profile.ps1"),
        ];

        for profile in &profiles {
            if profile.exists() {
                if let Ok(content) = fs::read_to_string(profile) {
                    let cleaned: Vec<&str> = content
                        .lines()
                        .filter(|line| {
                            !line.contains("yawcp.exe")
                                && !line.contains("function global:ping")
                                && !line.contains("function global:yawcp")
                        })
                        .collect();
                    let new_content = cleaned.join("\n").trim().to_string();
                    if new_content.is_empty() {
                        let _ = fs::remove_file(profile);
                    } else {
                        let _ = fs::write(profile, new_content);
                    }
                }
            }
        }
    }
}

fn setup_bash_profiles(dir: &Path) {
    let yawcp_exe = dir.join("yawcp.exe");
    let yawcp_str = yawcp_exe.to_string_lossy().replace('\\', "/");
    let alias_def = format!(
        "\nalias ping='\"{}\"'\nalias yawcp='\"{}\"'\n",
        yawcp_str, yawcp_str
    );

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile);
        for file in [p.join(".bashrc"), p.join(".bash_profile")] {
            let existing = fs::read_to_string(&file).unwrap_or_default();
            if !existing.contains("alias ping=") {
                let mut new_content = existing;
                new_content.push_str(&alias_def);
                let _ = fs::write(&file, new_content);
            }
        }
    }
}

fn remove_bash_profiles() {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile);
        for file in [p.join(".bashrc"), p.join(".bash_profile")] {
            if file.exists() {
                if let Ok(content) = fs::read_to_string(&file) {
                    let cleaned: Vec<&str> = content
                        .lines()
                        .filter(|line| {
                            !line.contains("yawcp.exe")
                                && !line.contains("alias ping=")
                                && !line.contains("alias yawcp=")
                        })
                        .collect();
                    let _ = fs::write(&file, cleaned.join("\n"));
                }
            }
        }
    }
}

fn setup_app_paths(dir: &Path) {
    let yawcp_exe = dir.join("yawcp.exe");
    let yawcp_str = yawcp_exe.to_string_lossy();

    let _ = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\yawcp.exe",
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &yawcp_str,
            "/f",
        ])
        .output();

    let _ = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\ping.exe",
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &yawcp_str,
            "/f",
        ])
        .output();
}

fn remove_app_paths() {
    let _ = Command::new("reg")
        .args([
            "delete",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\yawcp.exe",
            "/f",
        ])
        .output();

    let _ = Command::new("reg")
        .args([
            "delete",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\ping.exe",
            "/f",
        ])
        .output();
}

pub fn ensure_installed() {
    let appdata_dir = get_appdata_dir();
    let target_yawcp = appdata_dir.join("yawcp.exe");
    let autorun_cmd = appdata_dir.join("autorun.cmd");

    if !target_yawcp.exists() || !autorun_cmd.exists() {
        let _ = install_to_appdata();
    }
}
