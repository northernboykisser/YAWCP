mod api;
mod config;
mod flags;
mod models;
mod ui;
mod updater;

use std::io::{self, Write};
use std::time::{Duration, Instant};
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::api::{clean_host, create_client, fetch_target_geo, parse_results, poll_check_results, start_check};
use crate::config::{get_saved_language, load_config, save_language};
use crate::models::CheckType;
use crate::ui::{calculate_stats, render_header, render_nodes_table, select_language_interactive, select_post_action_interactive, PostCheckAction};
use crate::updater::{check_for_update, clean_old_binary, download_and_apply_update, find_matching_asset, prompt_update_interactive};

#[derive(Parser, Debug)]
#[command(name = "yawcp", version = "0.1.0")]
struct Cli {
    #[arg(value_name = "TARGET")]
    target: Option<String>,

    #[arg(short = 'c', long = "type", value_name = "TYPE")]
    check_type: Option<String>,

    #[arg(short = 'p', long = "port", value_name = "PORT")]
    port: Option<u16>,

    #[arg(short = 'l', long = "lang", value_name = "LANG")]
    lang: Option<String>,

    #[arg(short = 'n', long = "nodes", default_value_t = 15)]
    nodes: usize,

    #[arg(short = 't', long = "timeout", default_value_t = 20)]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    clean_old_binary();

    let cli = Cli::parse();

    let lang = match cli.lang {
        Some(l) => l,
        None => match get_saved_language() {
            Some(l) => l,
            None => {
                let selected = select_language_interactive();
                save_language(&selected);
                selected
            }
        },
    };

    let config = load_config(&lang);
    let client = create_client();

    if let Ok(Some(release)) = check_for_update(&client, &config.updater.github_repo).await {
        if prompt_update_interactive(&release, &config) {
            if let Some(asset) = find_matching_asset(&release.assets) {
                match download_and_apply_update(&client, &asset.browser_download_url, &config).await {
                    Ok(_) => {
                        println!("{}", config.updater.update_success.bright_green().bold());
                        return;
                    }
                    Err(e) => {
                        eprintln!("{} {}", config.updater.update_failed.red().bold(), e);
                    }
                }
            } else {
                eprintln!("{}", config.updater.no_asset_found.red().bold());
            }
        }
    }

    let mut current_target = cli.target.clone();
    let mut current_check_type = cli.check_type.clone().map(|s| match s.trim().to_lowercase().as_str() {
        "tcp" | "tcp port" => CheckType::Tcp,
        "udp" | "udp port" => CheckType::Udp,
        "dns" => CheckType::Dns,
        _ => CheckType::Ping,
    });

    'main_loop: loop {
        let raw_target = match current_target.take() {
            Some(t) if !t.trim().is_empty() => t,
            _ => {
                println!();
                print!("{}", config.prompts.target_prompt.bright_cyan().bold());
                io::stdout().flush().unwrap();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() || input.trim().is_empty() {
                    eprintln!("{}", config.errors.no_target.red().bold());
                    break 'main_loop;
                }
                input
            }
        };

        let mut target_host = clean_host(&raw_target);
        if target_host.is_empty() {
            eprintln!("{}", config.errors.invalid_host.red().bold());
            continue 'main_loop;
        }

        let check_type = match current_check_type.take() {
            Some(ct) => ct,
            None => select_check_type_interactive(&config.prompts),
        };

        if check_type == CheckType::Tcp || check_type == CheckType::Udp {
            if !target_host.contains(':') {
                let default_port = if let Some(p) = cli.port {
                    p
                } else if check_type == CheckType::Tcp {
                    443
                } else {
                    53
                };

                if cli.port.is_none() {
                    let prompt_text = config.prompts.port_prompt.replace("{}", &default_port.to_string());
                    print!("{}", prompt_text.bright_cyan().bold());
                    io::stdout().flush().unwrap();
                    let mut port_input = String::new();
                    let _ = io::stdin().read_line(&mut port_input);
                    let port_val = port_input.trim().parse::<u16>().unwrap_or(default_port);
                    target_host = format!("{}:{}", target_host, port_val);
                } else {
                    target_host = format!("{}:{}", target_host, default_port);
                }
            }
        }

        'retry_loop: loop {
            println!();

            let start_time = Instant::now();

            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            let start_msg = config.spinners.starting
                .replace("{type}", check_type.display_name(&config.prompts))
                .replace("{target}", &target_host);
            spinner.set_message(start_msg);
            spinner.enable_steady_tick(Duration::from_millis(80));

            let geo_future = fetch_target_geo(&client, &target_host);
            let check_future = start_check(&client, check_type, &target_host, cli.nodes, &config);

            let (mut geo_info, check_res) = tokio::join!(geo_future, check_future);

            let check_resp = match check_res {
                Ok(resp) => resp,
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("\n{} {}", config.errors.init_check_failed.red().bold(), err);
                    break 'retry_loop;
                }
            };

            let total_nodes = check_resp.nodes.len();
            let polling_init_msg = config.spinners.polling_init
                .replace("{total}", &total_nodes.to_string());
            spinner.set_message(polling_init_msg);

            let timeout_duration = Duration::from_secs(cli.timeout);
            let poll_interval = Duration::from_millis(1200);
            let mut raw_results = std::collections::HashMap::new();

            loop {
                if start_time.elapsed() >= timeout_duration {
                    break;
                }

                tokio::time::sleep(poll_interval).await;

                match poll_check_results(&client, &check_resp.request_id, &config).await {
                    Ok(results) => {
                        raw_results = results;
                        let parsed = parse_results(check_type, &check_resp.nodes, &raw_results, &config);
                        let completed_count = parsed.iter().filter(|n| n.is_completed).count();

                        let poll_msg = config.spinners.polling
                            .replace("{completed}", &completed_count.to_string())
                            .replace("{total}", &total_nodes.to_string());
                        spinner.set_message(poll_msg);

                        if completed_count >= total_nodes && total_nodes > 0 {
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }

            spinner.finish_and_clear();

            let final_nodes = parse_results(check_type, &check_resp.nodes, &raw_results, &config);

            let resolved_ip = final_nodes
                .iter()
                .find_map(|n| n.resolved_target_ip.as_deref());

            if geo_info.is_none() {
                if let Some(ip) = resolved_ip {
                    geo_info = fetch_target_geo(&client, ip).await;
                }
            }

            let stats = calculate_stats(&final_nodes);

            print!("\x1b[2J\x1b[1;1H");
            let _ = io::stdout().flush();

            render_header(
                check_type,
                &target_host,
                resolved_ip,
                geo_info.as_ref(),
                &stats,
                &config,
            );

            render_nodes_table(check_type, &final_nodes, &config);

            let action = select_post_action_interactive(check_resp.permanent_link.as_deref(), &config);
            match action {
                PostCheckAction::Retry => {
                    print!("\x1b[2J\x1b[1;1H");
                    let _ = io::stdout().flush();
                    continue 'retry_loop;
                }
                PostCheckAction::ToMenu => {
                    print!("\x1b[2J\x1b[1;1H");
                    let _ = io::stdout().flush();
                    continue 'main_loop;
                }
                PostCheckAction::OpenReport => {
                    print!("\x1b[2J\x1b[1;1H");
                    let _ = io::stdout().flush();
                    break 'main_loop;
                }
                PostCheckAction::Exit => {
                    print!("\x1b[2J\x1b[1;1H");
                    let _ = io::stdout().flush();
                    std::process::exit(0);
                }
            }
        }
    }
}

fn select_check_type_interactive(prompts: &crate::config::PromptsConfig) -> CheckType {
    use crossterm::event::{read, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::{stdout, Write};

    let options = [
        (prompts.menu_ping.as_str(), CheckType::Ping),
        (prompts.menu_tcp.as_str(), CheckType::Tcp),
        (prompts.menu_udp.as_str(), CheckType::Udp),
        (prompts.menu_dns.as_str(), CheckType::Dns),
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
        line.push_str(&prompts.check_type_title.bold().white().to_string());
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
    options[selected_idx].1
}
