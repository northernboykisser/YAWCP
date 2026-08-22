use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const RUS_JSON: &str = include_str!("../rus.json");
const ENG_JSON: &str = include_str!("../eng.json");

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub prompts: PromptsConfig,
    pub spinners: SpinnersConfig,
    pub header: HeaderConfig,
    pub table: TableConfig,
    pub buttons: ButtonsConfig,
    pub errors: ErrorsConfig,
    pub updater: UpdaterConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsConfig {
    pub target_prompt: String,
    pub port_prompt: String,
    pub check_type_title: String,
    pub menu_ping: String,
    pub menu_tcp: String,
    pub menu_udp: String,
    pub menu_dns: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpinnersConfig {
    pub starting: String,
    pub polling_init: String,
    pub polling: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeaderConfig {
    pub status_title: String,
    pub status_online: String,
    pub status_offline: String,
    pub status_partial: String,
    pub ping_label: String,
    pub rating_excellent: String,
    pub rating_good: String,
    pub rating_medium: String,
    pub rating_high: String,
    pub tcp_tested: String,
    pub dns_ok_badge: String,
    pub dns_fail_badge: String,
    pub unknown_country: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableConfig {
    pub col_location: String,
    pub col_status: String,
    pub col_ping: String,
    pub col_response_time: String,
    pub col_ttl: String,
    pub col_packets: String,
    pub col_ip_address: String,
    pub col_dns_records: String,
    pub badge_checking: String,
    pub badge_ok: String,
    pub badge_packet_loss: String,
    pub badge_timeout: String,
    pub badge_dns_fail: String,
    pub badge_open: String,
    pub badge_closed: String,
    pub badge_dns_ok: String,
    pub badge_nxdomain: String,
    pub msg_dns_error: String,
    pub msg_no_records: String,
    pub msg_waiting: String,
    pub msg_success: String,
    pub msg_error: String,
    pub placeholder_empty: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ButtonsConfig {
    pub open_report: String,
    pub to_menu: String,
    pub retry: String,
    pub close: String,
    pub opened_badge: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorsConfig {
    pub no_target: String,
    pub invalid_host: String,
    pub init_check_failed: String,
    pub connection_failed: String,
    pub http_status_error: String,
    pub json_parse_error: String,
    pub no_request_id: String,
    pub poll_failed: String,
    pub unknown_type_warning: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdaterConfig {
    pub github_repo: String,
    pub update_available_title: String,
    pub changelog_title: String,
    pub btn_update: String,
    pub btn_skip: String,
    pub downloading: String,
    pub update_success: String,
    pub update_failed: String,
    pub no_asset_found: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub lang: String,
}

pub fn get_saved_language() -> Option<String> {
    let path = find_file_path("settings.json");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                return Some(settings.lang);
            }
        }
    }
    None
}

pub fn save_language(lang: &str) {
    let path = find_file_path("settings.json");
    let settings = Settings {
        lang: lang.to_string(),
    };
    if let Ok(content) = serde_json::to_string_pretty(&settings) {
        let _ = fs::write(&path, content);
    }
}

pub fn load_config(lang: &str) -> AppConfig {
    let (file_name, default_json) = match lang.trim().to_lowercase().as_str() {
        "eng" | "en" | "english" => ("eng.json", ENG_JSON),
        _ => ("rus.json", RUS_JSON),
    };

    let path = find_file_path(file_name);
    if !path.exists() {
        let _ = fs::write(&path, default_json);
    }

    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
            return cfg;
        }
    }

    serde_json::from_str(default_json).unwrap_or_default()
}

fn find_file_path(file_name: &str) -> PathBuf {
    let local_path = Path::new(file_name);
    if local_path.exists() {
        return local_path.to_path_buf();
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_file = exe_dir.join(file_name);
            if exe_file.exists() {
                return exe_file;
            }
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        let appdata_dir = PathBuf::from(appdata).join("yawcp");
        let appdata_file = appdata_dir.join(file_name);
        if appdata_file.exists() {
            return appdata_file;
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA") {
        let appdata_dir = PathBuf::from(appdata).join("yawcp");
        let _ = fs::create_dir_all(&appdata_dir);
        return appdata_dir.join(file_name);
    }

    PathBuf::from(file_name)
}
