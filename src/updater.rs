use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::time::Duration;
use crate::config::AppConfig;

const UPDATER_USER_AGENT: &str = "yawcp-updater";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: Option<usize>,
}

pub fn clean_old_binary() {
    if let Ok(current_exe) = std::env::current_exe() {
        let old_exe = current_exe.with_extension("exe.old");
        if old_exe.exists() {
            let _ = fs::remove_file(old_exe);
        }
        #[cfg(unix)]
        {
            let old_unix = current_exe.with_extension("old");
            if old_unix.exists() {
                let _ = fs::remove_file(old_unix);
            }
        }
    }
}

pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse_ver = |v: &str| -> Vec<u64> {
        let clean = v.trim().trim_start_matches(|c| c == 'v' || c == 'V');
        clean
            .split('.')
            .filter_map(|part| {
                let num_str: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
                num_str.parse::<u64>().ok()
            })
            .collect()
    };

    let remote_parts = parse_ver(remote);
    let current_parts = parse_ver(current);

    if remote_parts.is_empty() || current_parts.is_empty() {
        return false;
    }

    for (r, c) in remote_parts.iter().zip(current_parts.iter()) {
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }

    remote_parts.len() > current_parts.len()
}

pub async fn check_for_update(client: &reqwest::Client, repo: &str) -> Result<Option<ReleaseInfo>, String> {
    let repo_clean = repo.trim();
    if repo_clean.is_empty() || repo_clean == "owner/yawcp" || !repo_clean.contains('/') {
        return Ok(None);
    }

    let url = format!("https://api.github.com/repos/{}/releases/latest", repo_clean);

    let res = client
        .get(&url)
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(USER_AGENT, UPDATER_USER_AGENT)
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Ok(None);
    }

    let release: ReleaseInfo = res.json().await.map_err(|e| e.to_string())?;

    let current_version = env!("CARGO_PKG_VERSION");
    if is_newer_version(&release.tag_name, current_version) {
        Ok(Some(release))
    } else {
        Ok(None)
    }
}

pub fn find_matching_asset<'a>(assets: &'a [ReleaseAsset]) -> Option<&'a ReleaseAsset> {
    if assets.is_empty() {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(asset) = assets.iter().find(|a| a.name.ends_with(".exe")) {
            return Some(asset);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(asset) = assets.iter().find(|a| a.name.to_lowercase().contains("linux")) {
            return Some(asset);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(asset) = assets.iter().find(|a| {
            let n = a.name.to_lowercase();
            n.contains("darwin") || n.contains("macos") || n.contains("apple")
        }) {
            return Some(asset);
        }
    }

    assets.first()
}

pub fn prompt_update_interactive(release: &ReleaseInfo, config: &AppConfig) -> bool {
    use crossterm::event::{read, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::stdout;

    let current_version = env!("CARGO_PKG_VERSION");
    let title_text = config.updater.update_available_title
        .replace("{}", current_version)
        .replacen("{}", &release.tag_name, 1);

    println!();
    println!("{}", format!("  {}  ", title_text).on_bright_blue().white().bold());
    println!();

    if let Some(body) = &release.body {
        let trimmed = body.trim();
        if !trimmed.is_empty() {
            println!("{}", config.updater.changelog_title.bold().bright_yellow());
            println!("{}", "─".repeat(50).dimmed());
            for line in trimmed.lines() {
                println!("  {}", line);
            }
            println!("{}", "─".repeat(50).dimmed());
            println!();
        }
    }

    let options = [
        (config.updater.btn_update.as_str(), true),
        (config.updater.btn_skip.as_str(), false),
    ];
    let mut selected_idx = 0;

    let _ = enable_raw_mode();
    let mut stdout = stdout();

    let render = |selected: usize, stdout: &mut std::io::Stdout| {
        let _ = crossterm::execute!(
            stdout,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
        );

        let mut line = String::new();
        line.push_str("  ");
        for (i, (label, _)) in options.iter().enumerate() {
            if i == selected {
                line.push_str(&format!("{} ", format!("[ {} ]", label).on_bright_cyan().black().bold()));
            } else {
                line.push_str(&format!("{} ", format!("[ {} ]", label).white().dimmed()));
            }
        }
        print!("{}", line);
        let _ = stdout.flush();
    };

    render(selected_idx, &mut stdout);

    let chosen = loop {
        if let Ok(Event::Key(key_event)) = read() {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }
            match key_event.code {
                KeyCode::Left | KeyCode::Up => {
                    if selected_idx == 0 {
                        selected_idx = options.len() - 1;
                    } else {
                        selected_idx -= 1;
                    }
                    render(selected_idx, &mut stdout);
                }
                KeyCode::Right | KeyCode::Down | KeyCode::Tab => {
                    if selected_idx + 1 >= options.len() {
                        selected_idx = 0;
                    } else {
                        selected_idx += 1;
                    }
                    render(selected_idx, &mut stdout);
                }
                KeyCode::Enter => {
                    break options[selected_idx].1;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    break false;
                }
                _ => {}
            }
        }
    };

    let _ = disable_raw_mode();
    let _ = crossterm::execute!(
        stdout,
        crossterm::cursor::MoveToColumn(0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
    );
    println!();

    chosen
}

pub async fn download_and_apply_update(
    client: &reqwest::Client,
    asset_url: &str,
    config: &AppConfig,
) -> Result<(), String> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(config.updater.downloading.clone());
    spinner.enable_steady_tick(Duration::from_millis(80));

    let res = client
        .get(asset_url)
        .header(USER_AGENT, UPDATER_USER_AGENT)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        spinner.finish_and_clear();
        return Err(format!("HTTP {}", res.status()));
    }

    let bytes = res.bytes().await.map_err(|e| e.to_string())?;

    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    let old_exe = current_exe.with_extension("exe.old");
    #[cfg(not(target_os = "windows"))]
    let old_exe = current_exe.with_extension("old");

    if old_exe.exists() {
        let _ = fs::remove_file(&old_exe);
    }

    fs::rename(&current_exe, &old_exe).map_err(|e| e.to_string())?;

    if let Err(e) = fs::write(&current_exe, bytes) {
        let _ = fs::rename(&old_exe, &current_exe);
        spinner.finish_and_clear();
        return Err(e.to_string());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&current_exe).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&current_exe, perms);
    }

    spinner.finish_and_clear();
    Ok(())
}
