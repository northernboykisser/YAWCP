use std::collections::HashMap;
use std::time::Duration;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde_json::Value;
use crate::config::AppConfig;
use crate::models::{CheckPingResponse, CheckType, NodeCheckResult, TargetGeoInfo};

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) YAWCP/0.1.0 (check-host CLI)";

pub fn clean_host(input: &str) -> String {
    let mut s = input.trim().to_lowercase();
    if let Some(stripped) = s.strip_prefix("https://") {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_prefix("http://") {
        s = stripped.to_string();
    }
    if let Some(idx) = s.find('/') {
        s = s[..idx].to_string();
    }
    s
}

pub fn create_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap_or_default()
}

pub async fn start_check(
    client: &reqwest::Client,
    check_type: CheckType,
    target: &str,
    max_nodes: usize,
    config: &AppConfig,
) -> Result<CheckPingResponse, String> {
    let url = format!(
        "https://check-host.net/{}?host={}&max_nodes={}",
        check_type.api_endpoint(),
        urlencoding(target),
        max_nodes
    );

    let res = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("{} {}", config.errors.connection_failed, e))?;

    if !res.status().is_success() {
        return Err(format!("{} {}", config.errors.http_status_error, res.status()));
    }

    let check_resp: CheckPingResponse = res
        .json()
        .await
        .map_err(|e| format!("{} {}", config.errors.json_parse_error, e))?;

    if check_resp.request_id.is_empty() {
        return Err(config.errors.no_request_id.clone());
    }

    Ok(check_resp)
}

pub async fn poll_check_results(
    client: &reqwest::Client,
    request_id: &str,
    config: &AppConfig,
) -> Result<HashMap<String, Value>, String> {
    let url = format!("https://check-host.net/check-result/{}", request_id);

    let res = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, DEFAULT_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("{} {}", config.errors.poll_failed, e))?;

    if !res.status().is_success() {
        return Err(format!("{} {}", config.errors.http_status_error, res.status()));
    }

    let results: HashMap<String, Value> = res
        .json()
        .await
        .map_err(|e| format!("{} {}", config.errors.json_parse_error, e))?;

    Ok(results)
}

pub fn parse_results(
    check_type: CheckType,
    nodes_info: &HashMap<String, Vec<String>>,
    raw_results: &HashMap<String, Value>,
    config: &AppConfig,
) -> Vec<NodeCheckResult> {
    let mut list = Vec::new();

    for (node_id, node_meta) in nodes_info {
        let country_code = node_meta.get(0).cloned().unwrap_or_default();
        let country_name = node_meta.get(1).cloned().unwrap_or_else(|| config.header.unknown_country.clone());
        let city = node_meta.get(2).cloned().unwrap_or_default();
        let node_ip = node_meta.get(3).cloned().unwrap_or_default();
        let asn = node_meta.get(4).cloned().unwrap_or_default();

        let mut is_completed = false;
        let mut is_success = false;
        let mut status_label = config.table.badge_checking.clone();
        let mut value_str = config.table.placeholder_empty.clone();
        let mut details_str = config.table.msg_waiting.clone();
        let mut avg_ms: Option<f64> = None;
        let mut resolved_target_ip: Option<String> = None;

        if let Some(val) = raw_results.get(node_id) {
            if !val.is_null() {
                is_completed = true;

                if let Some(arr) = val.as_array() {
                    let dns_fail = arr.iter().any(|item| {
                        if let Some(sub_arr) = item.as_array() {
                            sub_arr.iter().any(|v| v.is_null())
                        } else {
                            item.is_null()
                        }
                    });

                    if dns_fail {
                        status_label = config.table.badge_dns_fail.clone();
                        details_str = config.table.msg_dns_error.clone();
                    } else {
                        match check_type {
                            CheckType::Ping => {
                                let mut valid_pings = Vec::new();
                                let mut total_pings = 0;
                                let mut ok_pings = 0;
                                let mut dots = String::new();

                                for item in arr {
                                    if let Some(sub_arr) = item.as_array() {
                                        for attempt in sub_arr {
                                            if let Some(attempt_arr) = attempt.as_array() {
                                                total_pings += 1;
                                                let status = attempt_arr.get(0).and_then(|v| v.as_str()).unwrap_or("MALFORMED");
                                                let time_sec = attempt_arr.get(1).and_then(|v| v.as_f64());
                                                let ip_opt = attempt_arr.get(2).and_then(|v| v.as_str());

                                                if resolved_target_ip.is_none() && ip_opt.is_some() {
                                                    resolved_target_ip = ip_opt.map(|s| s.to_string());
                                                }

                                                if status == "OK" {
                                                    ok_pings += 1;
                                                    dots.push_str("● ");
                                                    if let Some(t) = time_sec {
                                                        valid_pings.push(t * 1000.0);
                                                    }
                                                } else {
                                                    dots.push_str("○ ");
                                                }
                                            }
                                        }
                                    }
                                }

                                if !valid_pings.is_empty() {
                                    let sum: f64 = valid_pings.iter().sum();
                                    let avg = sum / valid_pings.len() as f64;
                                    avg_ms = Some(avg);
                                    value_str = format!("{:.1} ms", avg);
                                }

                                if total_pings > 0 {
                                    details_str = format!("{}/{} {}", ok_pings, total_pings, dots.trim_end());
                                    if ok_pings == total_pings {
                                        is_success = true;
                                        status_label = config.table.badge_ok.clone();
                                    } else if ok_pings > 0 {
                                        is_success = true;
                                        status_label = config.table.badge_packet_loss.clone();
                                    } else {
                                        is_success = false;
                                        status_label = config.table.badge_timeout.clone();
                                    }
                                } else {
                                    status_label = config.table.badge_timeout.clone();
                                    details_str = config.table.msg_error.clone();
                                }
                            }
                            CheckType::Tcp | CheckType::Udp => {
                                if let Some(first) = arr.first() {
                                    if let Some(obj) = first.as_object() {
                                        if let Some(time_sec) = obj.get("time").and_then(|v| v.as_f64()) {
                                            let ms = time_sec * 1000.0;
                                            avg_ms = Some(ms);
                                            value_str = format!("{:.1} ms", ms);
                                            is_success = true;
                                            status_label = config.table.badge_open.clone();

                                            if let Some(addr) = obj.get("address").and_then(|v| v.as_str()) {
                                                details_str = addr.to_string();
                                                resolved_target_ip = Some(addr.to_string());
                                            } else {
                                                details_str = config.table.msg_success.clone();
                                            }
                                        } else if let Some(err) = obj.get("error").and_then(|v| v.as_str()) {
                                            is_success = false;
                                            if err.to_lowercase().contains("refused") {
                                                status_label = config.table.badge_closed.clone();
                                            } else {
                                                status_label = config.table.badge_timeout.clone();
                                            }
                                            details_str = err.to_string();
                                        }
                                    }
                                }
                            }
                            CheckType::Dns => {
                                if let Some(first) = arr.first() {
                                    if let Some(obj) = first.as_object() {
                                        let mut ips = Vec::new();
                                        if let Some(a_arr) = obj.get("A").and_then(|v| v.as_array()) {
                                            for a in a_arr {
                                                if let Some(ip) = a.as_str() {
                                                    ips.push(ip.to_string());
                                                    if resolved_target_ip.is_none() {
                                                        resolved_target_ip = Some(ip.to_string());
                                                    }
                                                }
                                            }
                                        }
                                        if let Some(aaaa_arr) = obj.get("AAAA").and_then(|v| v.as_array()) {
                                            for aaaa in aaaa_arr {
                                                if let Some(ip) = aaaa.as_str() {
                                                    ips.push(ip.to_string());
                                                }
                                            }
                                        }

                                        if let Some(ttl) = obj.get("TTL").and_then(|v| v.as_i64()) {
                                            value_str = format!("{}: {}", config.table.col_ttl, ttl);
                                        }

                                        if !ips.is_empty() {
                                            is_success = true;
                                            status_label = config.table.badge_dns_ok.clone();
                                            details_str = ips.join(", ");
                                        } else {
                                            is_success = false;
                                            status_label = config.table.badge_nxdomain.clone();
                                            details_str = config.table.msg_no_records.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        list.push(NodeCheckResult {
            node_id: node_id.clone(),
            country_code,
            country_name,
            city,
            node_ip,
            asn,
            is_completed,
            is_success,
            status_label,
            value_str,
            details_str,
            avg_ms,
            resolved_target_ip,
        });
    }

    list.sort_by(|a, b| {
        a.country_name
            .cmp(&b.country_name)
            .then_with(|| a.city.cmp(&b.city))
    });

    list
}

pub async fn fetch_target_geo(client: &reqwest::Client, target: &str) -> Option<TargetGeoInfo> {
    let host_only = if let Some(idx) = target.find(':') {
        &target[..idx]
    } else {
        target
    };

    let url = format!("http://ip-api.com/json/{}?fields=status,message,country,countryCode,region,regionName,city,isp,org,as,query", urlencoding(host_only));
    if let Ok(res) = client.get(&url).header(USER_AGENT, DEFAULT_USER_AGENT).send().await {
        if let Ok(geo) = res.json::<TargetGeoInfo>().await {
            if geo.status == "success" {
                return Some(geo);
            }
        }
    }

    let fallback_url = format!("https://ipwho.is/{}", urlencoding(host_only));
    if let Ok(res) = client.get(&fallback_url).header(USER_AGENT, DEFAULT_USER_AGENT).send().await {
        if let Ok(val) = res.json::<Value>().await {
            if val.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                return Some(TargetGeoInfo {
                    query: val.get("ip").and_then(|v| v.as_str()).unwrap_or(host_only).to_string(),
                    status: "success".to_string(),
                    country: val.get("country").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    country_code: val.get("country_code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    region: val.get("region_code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    region_name: val.get("region").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    city: val.get("city").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    isp: val.get("connection").and_then(|c| c.get("isp")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    org: val.get("connection").and_then(|c| c.get("org")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    as_info: val.get("connection").and_then(|c| c.get("asn")).and_then(|v| v.as_i64()).map(|n| format!("AS{}", n)).unwrap_or_default(),
                });
            }
        }
    }

    None
}

fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}
