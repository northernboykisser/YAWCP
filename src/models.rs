use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckType {
    Ping,
    Tcp,
    Udp,
    Dns,
}

impl CheckType {
    pub fn api_endpoint(&self) -> &'static str {
        match self {
            CheckType::Ping => "check-ping",
            CheckType::Tcp => "check-tcp",
            CheckType::Udp => "check-udp",
            CheckType::Dns => "check-dns",
        }
    }

    pub fn display_name<'a>(&self, config: &'a crate::config::PromptsConfig) -> &'a str {
        match self {
            CheckType::Ping => &config.menu_ping,
            CheckType::Tcp => &config.menu_tcp,
            CheckType::Udp => &config.menu_udp,
            CheckType::Dns => &config.menu_dns,
        }
    }

    pub fn value_header<'a>(&self, config: &'a crate::config::TableConfig) -> &'a str {
        match self {
            CheckType::Ping => &config.col_ping,
            CheckType::Tcp | CheckType::Udp => &config.col_response_time,
            CheckType::Dns => &config.col_ttl,
        }
    }

    pub fn details_header<'a>(&self, config: &'a crate::config::TableConfig) -> &'a str {
        match self {
            CheckType::Ping => &config.col_packets,
            CheckType::Tcp | CheckType::Udp => &config.col_ip_address,
            CheckType::Dns => &config.col_dns_records,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckPingResponse {
    pub ok: Option<i64>,
    pub request_id: String,
    pub permanent_link: Option<String>,
    #[serde(default)]
    pub nodes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetGeoInfo {
    pub query: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub country: String,
    #[serde(rename = "countryCode", default)]
    pub country_code: String,
    #[serde(default)]
    pub region: String,
    #[serde(rename = "regionName", default)]
    pub region_name: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub isp: String,
    #[serde(default)]
    pub org: String,
    #[serde(rename = "as", default)]
    pub as_info: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PingAttempt {
    pub status: String,
    pub time_ms: Option<f64>,
    pub ip: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NodeCheckResult {
    pub node_id: String,
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    pub node_ip: String,
    pub asn: String,
    pub is_completed: bool,
    pub is_success: bool,
    pub status_label: String,
    pub value_str: String,
    pub details_str: String,
    pub avg_ms: Option<f64>,
    pub resolved_target_ip: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OverallStats {
    pub total_nodes: usize,
    pub completed_nodes: usize,
    pub successful_nodes: usize,
    pub failed_nodes: usize,
    pub avg_ms: Option<f64>,
    pub is_online: bool,
    pub is_partial: bool,
}
