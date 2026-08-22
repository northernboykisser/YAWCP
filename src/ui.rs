use colored::*;
use unicode_width::UnicodeWidthChar;
use crate::flags::{get_country_emoji, get_flag_art};
use crate::models::{CheckType, NodeCheckResult, OverallStats, TargetGeoInfo};

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn visible_width(s: &str) -> usize {
    let plain = strip_ansi(s);
    let mut width = 0;
    let mut chars = plain.chars().peekable();
    while let Some(c) = chars.next() {
        if ('\u{1F1E6}'..='\u{1F1FF}').contains(&c) {
            if let Some(&next_c) = chars.peek() {
                if ('\u{1F1E6}'..='\u{1F1FF}').contains(&next_c) {
                    chars.next();
                    width += 2;
                    continue;
                }
            }
            width += 2;
        } else if c == '●' || c == '○' {
            width += 1;
        } else {
            width += UnicodeWidthChar::width(c).unwrap_or(1);
        }
    }
    width
}

fn pad_right(s: &str, target_width: usize) -> String {
    let current_width = visible_width(s);
    if current_width >= target_width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target_width - current_width))
    }
}

fn pad_center(s: &str, target_width: usize) -> String {
    let current_width = visible_width(s);
    if current_width >= target_width {
        s.to_string()
    } else {
        let total_pad = target_width - current_width;
        let left_pad = total_pad / 2;
        let right_pad = total_pad - left_pad;
        format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
    }
}

pub fn format_ping_colored(ms: Option<f64>) -> String {
    match ms {
        Some(val) if val < 50.0 => format!("{:.1} ms", val).bright_green().bold().to_string(),
        Some(val) if val < 100.0 => format!("{:.1} ms", val).green().bold().to_string(),
        Some(val) if val < 200.0 => format!("{:.1} ms", val).yellow().bold().to_string(),
        Some(val) if val < 300.0 => format!("{:.1} ms", val).bright_red().bold().to_string(),
        Some(val) => format!("{:.1} ms", val).red().bold().to_string(),
        None => "—".dimmed().to_string(),
    }
}

pub fn calculate_stats(nodes: &[NodeCheckResult]) -> OverallStats {
    let total_nodes = nodes.len();
    let completed_nodes = nodes.iter().filter(|n| n.is_completed).count();
    let successful_nodes = nodes.iter().filter(|n| n.is_success).count();
    let failed_nodes = nodes.iter().filter(|n| n.is_completed && !n.is_success).count();

    let all_valid_pings: Vec<f64> = nodes
        .iter()
        .filter(|n| n.is_success && n.avg_ms.is_some())
        .filter_map(|n| n.avg_ms)
        .collect();

    let avg_ms = if !all_valid_pings.is_empty() {
        let sum: f64 = all_valid_pings.iter().sum();
        Some(sum / all_valid_pings.len() as f64)
    } else {
        None
    };

    let is_online = successful_nodes > 0 && failed_nodes == 0;
    let is_partial = successful_nodes > 0 && failed_nodes > 0;

    OverallStats {
        total_nodes,
        completed_nodes,
        successful_nodes,
        failed_nodes,
        avg_ms,
        is_online,
        is_partial,
    }
}

pub fn render_header(
    check_type: CheckType,
    target_host: &str,
    resolved_ip: Option<&str>,
    geo_info: Option<&TargetGeoInfo>,
    stats: &OverallStats,
    config: &crate::config::AppConfig,
) {
    let country_code = geo_info
        .map(|g| g.country_code.as_str())
        .unwrap_or("??");
    let country_name = geo_info
        .map(|g| g.country.as_str())
        .unwrap_or(&config.header.unknown_country);
    let city_name = geo_info
        .map(|g| g.city.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let isp_info = geo_info
        .map(|g| {
            if !g.isp.is_empty() {
                g.isp.as_str()
            } else if !g.org.is_empty() {
                g.org.as_str()
            } else {
                g.as_info.as_str()
            }
        })
        .unwrap_or("");

    let flag_lines = get_flag_art(country_code);

    let ip_str = resolved_ip
        .or_else(|| geo_info.map(|g| g.query.as_str()))
        .unwrap_or(&config.table.placeholder_empty);

    let right_line1 = format!(
        "{}  {}  {}",
        "🎯".bright_cyan(),
        target_host.bright_white().bold(),
        format!("[ {} ]", ip_str).cyan()
    );

    let emoji_flag = get_country_emoji(country_code);
    let location_text = if !city_name.is_empty() {
        if !isp_info.is_empty() {
            format!("{} {}, {}  {}", emoji_flag, country_name, city_name, format!("({})", isp_info).dimmed())
        } else {
            format!("{} {}, {}", emoji_flag, country_name, city_name)
        }
    } else if !isp_info.is_empty() {
        format!("{} {}  {}", emoji_flag, country_name, format!("({})", isp_info).dimmed())
    } else {
        format!("{} {}", emoji_flag, country_name)
    };
    let right_line2 = format!("{}  {}", "📍".bright_yellow(), location_text.white());

    let status_badge = if stats.successful_nodes == 0 {
        format!(" {} ", config.header.status_offline).on_red().bright_white().bold()
    } else if stats.is_partial {
        format!(" {} ", config.header.status_partial).on_yellow().black().bold()
    } else {
        format!(" {} ", config.header.status_online).on_green().bright_white().bold()
    };
    let right_line3 = format!("{}  {}", config.header.status_title.bold().white(), status_badge);

    let right_line4 = match check_type {
        CheckType::Ping => {
            let ping_val = format_ping_colored(stats.avg_ms);
            let rating = if let Some(avg) = stats.avg_ms {
                if avg < 50.0 {
                    config.header.rating_excellent.bright_green()
                } else if avg < 100.0 {
                    config.header.rating_good.green()
                } else if avg < 200.0 {
                    config.header.rating_medium.yellow()
                } else {
                    config.header.rating_high.bright_red()
                }
            } else {
                "".normal()
            };
            format!("{}  {}  {}", config.header.ping_label.bold().white(), ping_val, rating)
        }
        CheckType::Tcp | CheckType::Udp => {
            let val = format_ping_colored(stats.avg_ms);
            format!("{}  {}  {}", format!("{}:", check_type.display_name(&config.prompts)).bold().white(), val, config.header.tcp_tested.green())
        }
        CheckType::Dns => {
            format!("{}  {}", "DNS:".bold().white(), if stats.successful_nodes > 0 { config.header.dns_ok_badge.bright_green() } else { config.header.dns_fail_badge.red() })
        }
    };

    let right_lines = [
        right_line1,
        right_line2,
        right_line3,
        right_line4,
        "".to_string(),
        "".to_string(),
    ];

    let max_right_width = right_lines
        .iter()
        .map(|l| visible_width(l))
        .max()
        .unwrap_or(50)
        .max(45);

    let total_inner_width = 16 + 3 + max_right_width + 2;

    println!();
    println!("╭{}╮", "─".repeat(total_inner_width).dimmed());

    for i in 0..6 {
        let flag_part = flag_lines.get(i).map(|s| s.as_str()).unwrap_or("                ");
        let info_part = right_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let padded_info = pad_right(info_part, max_right_width);
        println!("│  {}   {}  │", flag_part, padded_info);
    }

    println!("╰{}╯", "─".repeat(total_inner_width).dimmed());
    println!();
}

pub fn render_nodes_table(check_type: CheckType, nodes: &[NodeCheckResult], config: &crate::config::AppConfig) {
    struct TableRow {
        loc: String,
        status: String,
        value: String,
        details: String,
    }

    let header_col1 = &config.table.col_location;
    let header_col2 = &config.table.col_status;
    let header_col3 = check_type.value_header(&config.table);
    let header_col4 = check_type.details_header(&config.table);

    let mut rows = Vec::new();

    for node in nodes {
        let emoji = get_country_emoji(&node.country_code);
        let loc = format!("{} {}, {}", emoji, node.country_name, node.city);

        let status = if !node.is_completed {
            format!(" {} ", config.table.badge_checking).yellow().to_string()
        } else if node.is_success {
            if node.status_label == config.table.badge_packet_loss {
                format!(" {} ", config.table.badge_packet_loss).on_yellow().black().bold().to_string()
            } else if node.status_label == config.table.badge_open
                || node.status_label == config.table.badge_ok
                || node.status_label == config.table.badge_dns_ok {
                format!(" {} ", node.status_label).on_green().bright_white().bold().to_string()
            } else {
                format!(" {} ", node.status_label).green().to_string()
            }
        } else {
            format!(" {} ", node.status_label).on_red().bright_white().bold().to_string()
        };

        let value = match check_type {
            CheckType::Ping | CheckType::Tcp | CheckType::Udp => {
                if node.avg_ms.is_some() {
                    format_ping_colored(node.avg_ms)
                } else {
                    node.value_str.clone().dimmed().to_string()
                }
            }
            CheckType::Dns => node.value_str.clone(),
        };

        let details = if check_type == CheckType::Ping {
            if node.details_str.contains('●') || node.details_str.contains('○') {
                let mut colored_dots = String::new();
                for part in node.details_str.split_whitespace() {
                    if part.contains('/') {
                        colored_dots.push_str(&part.cyan().to_string());
                        colored_dots.push(' ');
                    } else if part == "●" {
                        colored_dots.push_str(&"● ".bright_green().to_string());
                    } else if part == "○" {
                        colored_dots.push_str(&"○ ".bright_red().to_string());
                    }
                }
                colored_dots.trim_end().to_string()
            } else {
                node.details_str.clone()
            }
        } else {
            node.details_str.clone()
        };

        rows.push(TableRow {
            loc,
            status,
            value,
            details,
        });
    }

    let max_loc_len = rows.iter().map(|r| visible_width(&r.loc)).max().unwrap_or(0);
    let max_status_len = rows.iter().map(|r| visible_width(&r.status)).max().unwrap_or(0);
    let max_value_len = rows.iter().map(|r| visible_width(&r.value)).max().unwrap_or(0);
    let max_details_len = rows.iter().map(|r| visible_width(&r.details)).max().unwrap_or(0);

    let col1_w = max_loc_len.max(visible_width(header_col1));
    let col2_w = max_status_len.max(visible_width(header_col2));
    let col3_w = max_value_len.max(visible_width(header_col3));
    let col4_w = max_details_len.max(visible_width(header_col4));

    let top_border = format!(
        "┌{}┬{}┬{}┬{}┐",
        "─".repeat(col1_w + 2),
        "─".repeat(col2_w + 2),
        "─".repeat(col3_w + 2),
        "─".repeat(col4_w + 2)
    );
    let mid_border = format!(
        "├{}┼{}┼{}┼{}┤",
        "─".repeat(col1_w + 2),
        "─".repeat(col2_w + 2),
        "─".repeat(col3_w + 2),
        "─".repeat(col4_w + 2)
    );
    let bot_border = format!(
        "└{}┴{}┴{}┴{}┘",
        "─".repeat(col1_w + 2),
        "─".repeat(col2_w + 2),
        "─".repeat(col3_w + 2),
        "─".repeat(col4_w + 2)
    );

    println!("{}", top_border.cyan());
    println!(
        "│ {} │ {} │ {} │ {} │",
        pad_center(header_col1, col1_w).bold().white(),
        pad_center(header_col2, col2_w).bold().white(),
        pad_center(header_col3, col3_w).bold().white(),
        pad_center(header_col4, col4_w).bold().white()
    );
    println!("{}", mid_border.cyan());

    for row in rows {
        println!(
            "│ {} │ {} │ {} │ {} │",
            pad_right(&row.loc, col1_w),
            pad_center(&row.status, col2_w),
            pad_center(&row.value, col3_w),
            pad_right(&row.details, col4_w)
        );
    }

    println!("{}", bot_border.cyan());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCheckAction {
    OpenReport,
    ToMenu,
    Retry,
    Exit,
}

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

pub fn select_language_interactive() -> String {
    use crossterm::event::{read, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::{stdout, Write};

    let options = [("RUS", "rus"), ("ENG", "eng")];
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

    println!();
    render(selected_idx, &mut stdout);

    loop {
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
                    break;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    let _ = disable_raw_mode();
                    let _ = crossterm::execute!(
                        stdout,
                        crossterm::cursor::MoveToColumn(0),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                    );
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = crossterm::execute!(
        stdout,
        crossterm::cursor::MoveToColumn(0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
    );
    options[selected_idx].1.to_string()
}

pub fn select_post_action_interactive(permanent_link: Option<&str>, config: &crate::config::AppConfig) -> PostCheckAction {
    use crossterm::event::{read, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::{stdout, Write};

    let options = [
        (config.buttons.open_report.as_str(), PostCheckAction::OpenReport),
        (config.buttons.to_menu.as_str(), PostCheckAction::ToMenu),
        (config.buttons.retry.as_str(), PostCheckAction::Retry),
        (config.buttons.close.as_str(), PostCheckAction::Exit),
    ];
    let mut selected_idx = 0;
    let mut opened = false;

    let _ = enable_raw_mode();
    let mut stdout = stdout();

    let render = |selected: usize, stdout: &mut std::io::Stdout, was_opened: bool| {
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
        if was_opened {
            line.push_str(&config.buttons.opened_badge.bright_green().bold().to_string());
        }
        print!("{}", line);
        let _ = stdout.flush();
    };

    println!();
    render(selected_idx, &mut stdout, opened);

    loop {
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
                    render(selected_idx, &mut stdout, opened);
                }
                KeyCode::Right | KeyCode::Down | KeyCode::Tab => {
                    if selected_idx + 1 >= options.len() {
                        selected_idx = 0;
                    } else {
                        selected_idx += 1;
                    }
                    render(selected_idx, &mut stdout, opened);
                }
                KeyCode::Enter => {
                    let action = options[selected_idx].1;
                    if action == PostCheckAction::OpenReport {
                        if let Some(link) = permanent_link {
                            open_browser(link);
                            opened = true;
                            render(selected_idx, &mut stdout, opened);
                            continue;
                        }
                    }
                    let _ = disable_raw_mode();
                    let _ = crossterm::execute!(
                        stdout,
                        crossterm::cursor::MoveToColumn(0),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                    );
                    return action;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    let _ = disable_raw_mode();
                    let _ = crossterm::execute!(
                        stdout,
                        crossterm::cursor::MoveToColumn(0),
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
                    );
                    return PostCheckAction::Exit;
                }
                _ => {}
            }
        }
    }
}
