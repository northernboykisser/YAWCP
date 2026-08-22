pub fn get_country_emoji(code: &str) -> String {
    let code = code.trim().to_uppercase();
    if code.len() == 2 {
        let mut emoji = String::new();
        for c in code.chars() {
            if ('A'..='Z').contains(&c) {
                let offset = (c as u32) - ('A' as u32);
                if let Some(unicode_char) = char::from_u32(0x1F1E6 + offset) {
                    emoji.push(unicode_char);
                }
            }
        }
        if !emoji.is_empty() {
            return emoji;
        }
    }
    "🌐".to_string()
}

fn rgb_block(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

fn rgb_bg(bg_r: u8, bg_g: u8, bg_b: u8, fg_r: u8, fg_g: u8, fg_b: u8, text: &str) -> String {
    format!(
        "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}\x1b[0m",
        bg_r, bg_g, bg_b, fg_r, fg_g, fg_b, text
    )
}

pub fn get_flag_art(code: &str) -> Vec<String> {
    let c = code.trim().to_uppercase();
    match c.as_str() {
        "RU" => flag_russia(),
        "US" => flag_usa(),
        "DE" => flag_germany(),
        "FR" => flag_france(),
        "UA" => flag_ukraine(),
        "GB" | "UK" => flag_uk(),
        "NL" => flag_netherlands(),
        "JP" => flag_japan(),
        "CN" => flag_china(),
        "PL" => flag_poland(),
        "IT" => flag_italy(),
        "ES" => flag_spain(),
        "CA" => flag_canada(),
        "CH" => flag_switzerland(),
        "SE" => flag_sweden(),
        "FI" => flag_finland(),
        "NO" => flag_norway(),
        "TR" => flag_turkey(),
        "KZ" => flag_kazakhstan(),
        "BY" => flag_belarus(),
        "BR" => flag_brazil(),
        "IN" => flag_india(),
        "KR" => flag_south_korea(),
        "SG" => flag_singapore(),
        "HK" => flag_hong_kong(),
        "IE" => flag_ireland(),
        "AU" => flag_australia(),
        "AT" => flag_austria(),
        "BE" => flag_belgium(),
        "CZ" => flag_czech(),
        "DK" => flag_denmark(),
        "GR" => flag_greece(),
        "HU" => flag_hungary(),
        "IL" => flag_israel(),
        "PT" => flag_portugal(),
        "RO" => flag_romania(),
        "AR" => flag_argentina(),
        _ => flag_generic(&c),
    }
}

fn flag_russia() -> Vec<String> {
    let w = rgb_block(255, 255, 255, "████████████████");
    let b = rgb_block(0, 57, 166, "████████████████");
    let r = rgb_block(213, 43, 30, "████████████████");
    vec![w.clone(), w, b.clone(), b, r.clone(), r]
}

fn flag_usa() -> Vec<String> {
    let star_line1 = rgb_bg(60, 59, 110, 255, 255, 255, " ★ ★ ★ ") + &rgb_block(178, 34, 52, "█████████");
    let star_line2 = rgb_bg(60, 59, 110, 255, 255, 255, "  ★ ★  ") + &rgb_block(255, 255, 255, "█████████");
    let star_line3 = rgb_bg(60, 59, 110, 255, 255, 255, " ★ ★ ★ ") + &rgb_block(178, 34, 52, "█████████");
    let stripe_w = rgb_block(255, 255, 255, "████████████████");
    let stripe_r = rgb_block(178, 34, 52, "████████████████");
    vec![star_line1, star_line2, star_line3, stripe_w.clone(), stripe_r, stripe_w]
}

fn flag_germany() -> Vec<String> {
    let k = rgb_block(35, 35, 35, "████████████████");
    let r = rgb_block(221, 0, 0, "████████████████");
    let y = rgb_block(255, 206, 0, "████████████████");
    vec![k.clone(), k, r.clone(), r, y.clone(), y]
}

fn flag_france() -> Vec<String> {
    let b = rgb_block(0, 38, 84, "█████");
    let w = rgb_block(255, 255, 255, "██████");
    let r = rgb_block(237, 41, 57, "█████");
    let line = format!("{}{}{}", b, w, r);
    vec![
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line,
    ]
}

fn flag_ukraine() -> Vec<String> {
    let b = rgb_block(0, 87, 183, "████████████████");
    let y = rgb_block(255, 215, 0, "████████████████");
    vec![b.clone(), b.clone(), b, y.clone(), y.clone(), y]
}

fn flag_uk() -> Vec<String> {
    let b = "\x1b[38;2;1;33;105m";
    let w = "\x1b[38;2;255;255;255m";
    let r = "\x1b[38;2;200;16;46m";
    let rst = "\x1b[0m";

    vec![
        format!("{}█{}██{}█{}██{}█{}██{}█{}██{}█{}", r, w, b, w, r, w, b, w, r, rst),
        format!("{}█{}██{}█{}██{}█{}██{}█{}██{}█{}", b, r, w, w, r, w, w, r, b, rst),
        format!("{}████████████████{}", r, rst),
        format!("{}████████████████{}", r, rst),
        format!("{}█{}██{}█{}██{}█{}██{}█{}██{}█{}", b, r, w, w, r, w, w, r, b, rst),
        format!("{}█{}██{}█{}██{}█{}██{}█{}██{}█{}", r, w, b, w, r, w, b, w, r, rst),
    ]
}

fn flag_netherlands() -> Vec<String> {
    let r = rgb_block(174, 28, 40, "████████████████");
    let w = rgb_block(255, 255, 255, "████████████████");
    let b = rgb_block(33, 70, 139, "████████████████");
    vec![r.clone(), r, w.clone(), w, b.clone(), b]
}

fn flag_japan() -> Vec<String> {
    let w = rgb_block(255, 255, 255, "████████████████");
    let w5 = rgb_block(255, 255, 255, "█████");
    let w4 = rgb_block(255, 255, 255, "████");
    let r6 = rgb_block(188, 0, 45, "██████");
    let r8 = rgb_block(188, 0, 45, "████████");

    vec![
        w.clone(),
        format!("{}{}{}", w5, r6, w5),
        format!("{}{}{}", w4, r8, w4),
        format!("{}{}{}", w4, r8, w4),
        format!("{}{}{}", w5, r6, w5),
        w,
    ]
}

fn flag_china() -> Vec<String> {
    let r_full = rgb_block(222, 41, 16, "████████████████");
    let line1 = rgb_bg(222, 41, 16, 255, 222, 0, " ★            ") + &rgb_block(222, 41, 16, "  ");
    let line2 = rgb_bg(222, 41, 16, 255, 222, 0, "   •   •      ") + &rgb_block(222, 41, 16, "  ");
    let line3 = rgb_bg(222, 41, 16, 255, 222, 0, "   •   •      ") + &rgb_block(222, 41, 16, "  ");
    vec![line1, line2, line3, r_full.clone(), r_full.clone(), r_full]
}

fn flag_poland() -> Vec<String> {
    let w = rgb_block(255, 255, 255, "████████████████");
    let r = rgb_block(220, 20, 60, "████████████████");
    vec![w.clone(), w.clone(), w, r.clone(), r.clone(), r]
}

fn flag_italy() -> Vec<String> {
    let g = rgb_block(0, 140, 69, "█████");
    let w = rgb_block(255, 255, 255, "██████");
    let r = rgb_block(205, 33, 42, "█████");
    let line = format!("{}{}{}", g, w, r);
    vec![
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line,
    ]
}

fn flag_spain() -> Vec<String> {
    let r = rgb_block(170, 21, 27, "████████████████");
    let y = rgb_block(241, 191, 0, "████████████████");
    let y_coat = rgb_bg(241, 191, 0, 170, 21, 27, "   [⚜]          ") + &rgb_block(241, 191, 0, "");
    vec![r.clone(), y.clone(), y_coat, y, r.clone(), r]
}

fn flag_canada() -> Vec<String> {
    let r = rgb_block(216, 6, 33, "████");
    let w = rgb_block(255, 255, 255, "████████");
    let w_leaf1 = rgb_bg(255, 255, 255, 216, 6, 33, "   🍁   ");
    let w_leaf2 = rgb_bg(255, 255, 255, 216, 6, 33, "   |    ");
    vec![
        format!("{}{}{}", r, w, r),
        format!("{}{}{}", r, w_leaf1, r),
        format!("{}{}{}", r, w_leaf1, r),
        format!("{}{}{}", r, w_leaf2, r),
        format!("{}{}{}", r, w, r),
        format!("{}{}{}", r, w, r),
    ]
}

fn flag_switzerland() -> Vec<String> {
    let r_full = rgb_block(218, 41, 28, "████████████████");
    let r5 = rgb_block(218, 41, 28, "██████");
    let w4 = rgb_block(255, 255, 255, "████");
    let r3 = rgb_block(218, 41, 28, "███");
    let w10 = rgb_block(255, 255, 255, "██████████");

    vec![
        r_full.clone(),
        format!("{}{}{}", r5, w4, r5),
        format!("{}{}{}", r3, w10, r3),
        format!("{}{}{}", r3, w10, r3),
        format!("{}{}{}", r5, w4, r5),
        r_full,
    ]
}

fn flag_sweden() -> Vec<String> {
    let b4 = rgb_block(0, 106, 167, "████");
    let y3 = rgb_block(254, 204, 0, "███");
    let b9 = rgb_block(0, 106, 167, "█████████");
    let y_full = rgb_block(254, 204, 0, "████████████████");

    let line_cross = format!("{}{}{}", b4, y3, b9);
    vec![
        line_cross.clone(),
        line_cross.clone(),
        y_full.clone(),
        y_full,
        line_cross.clone(),
        line_cross,
    ]
}

fn flag_finland() -> Vec<String> {
    let w4 = rgb_block(255, 255, 255, "████");
    let b3 = rgb_block(0, 47, 108, "███");
    let w9 = rgb_block(255, 255, 255, "█████████");
    let b_full = rgb_block(0, 47, 108, "████████████████");

    let line = format!("{}{}{}", w4, b3, w9);
    vec![
        line.clone(),
        line.clone(),
        b_full.clone(),
        b_full,
        line.clone(),
        line,
    ]
}

fn flag_norway() -> Vec<String> {
    let r3 = rgb_block(186, 12, 47, "███");
    let w1 = rgb_block(255, 255, 255, "█");
    let b2 = rgb_block(0, 32, 91, "██");
    let r9 = rgb_block(186, 12, 47, "█████████");
    let w_full = rgb_block(255, 255, 255, "████████████████");
    let b_full = rgb_block(0, 32, 91, "████████████████");

    let line = format!("{}{}{}{}{}", r3, w1, b2, w1, r9);
    vec![
        line.clone(),
        line.clone(),
        w_full,
        b_full,
        line.clone(),
        line,
    ]
}

fn flag_turkey() -> Vec<String> {
    let r = rgb_block(227, 10, 23, "████████████████");
    let c1 = rgb_bg(227, 10, 23, 255, 255, 255, "    ╭─╮         ") + &rgb_block(227, 10, 23, "");
    let c2 = rgb_bg(227, 10, 23, 255, 255, 255, "   │ ☾ ★        ") + &rgb_block(227, 10, 23, "");
    let c3 = rgb_bg(227, 10, 23, 255, 255, 255, "    ╰─╯         ") + &rgb_block(227, 10, 23, "");
    vec![r.clone(), c1, c2, c3, r.clone(), r]
}

fn flag_kazakhstan() -> Vec<String> {
    let c = rgb_block(0, 175, 202, "████████████████");
    let orn = rgb_bg(0, 175, 202, 254, 209, 0, "§ ");
    let sun1 = rgb_bg(0, 175, 202, 254, 209, 0, "   ☼☼   ") + &rgb_block(0, 175, 202, "██████");
    let sun2 = rgb_bg(0, 175, 202, 254, 209, 0, "  ~🦅~  ") + &rgb_block(0, 175, 202, "██████");
    vec![
        c.clone(),
        format!("{}{}", orn, sun1),
        format!("{}{}", orn, sun2),
        c.clone(),
        c.clone(),
        c,
    ]
}

fn flag_belarus() -> Vec<String> {
    let orn = rgb_bg(255, 255, 255, 195, 33, 38, "░▓");
    let r = rgb_block(195, 33, 38, "██████████████");
    let g = rgb_block(0, 122, 60, "██████████████");

    vec![
        format!("{}{}", orn, r),
        format!("{}{}", orn, r),
        format!("{}{}", orn, r),
        format!("{}{}", orn, r),
        format!("{}{}", orn, g),
        format!("{}{}", orn, g),
    ]
}

fn flag_brazil() -> Vec<String> {
    let g = rgb_block(0, 156, 59, "████████████████");
    let l2 = rgb_bg(0, 156, 59, 255, 223, 0, "    ◢████◣      ");
    let l3 = rgb_bg(0, 156, 59, 255, 223, 0, "  ◢██") + &rgb_bg(0, 39, 118, 255, 255, 255, "●●●●") + &rgb_bg(0, 156, 59, 255, 223, 0, "██◣  ");
    let l4 = rgb_bg(0, 156, 59, 255, 223, 0, "  ◥██") + &rgb_bg(0, 39, 118, 255, 255, 255, "●●●●") + &rgb_bg(0, 156, 59, 255, 223, 0, "██◤  ");
    let l5 = rgb_bg(0, 156, 59, 255, 223, 0, "    ◥████◤      ");
    vec![g.clone(), l2, l3, l4, l5, g]
}

fn flag_india() -> Vec<String> {
    let s = rgb_block(255, 103, 31, "████████████████");
    let w_chakra1 = rgb_bg(255, 255, 255, 0, 0, 128, "     (☸)        ") + &rgb_block(255, 255, 255, "");
    let w_chakra2 = rgb_bg(255, 255, 255, 0, 0, 128, "     (☸)        ") + &rgb_block(255, 255, 255, "");
    let g = rgb_block(4, 106, 56, "████████████████");
    vec![s.clone(), s, w_chakra1, w_chakra2, g.clone(), g]
}

fn flag_south_korea() -> Vec<String> {
    let w = rgb_block(255, 255, 255, "████████████████");
    let l2 = rgb_bg(255, 255, 255, 0, 0, 0, " ☰  ") + &rgb_bg(255, 255, 255, 205, 46, 58, "◢██◣") + &rgb_bg(255, 255, 255, 0, 0, 0, "  ☵ ");
    let l3 = rgb_bg(255, 255, 255, 0, 0, 0, "    ") + &rgb_bg(255, 255, 255, 0, 71, 160, "◥██◤") + &rgb_bg(255, 255, 255, 0, 0, 0, "    ");
    let l4 = rgb_bg(255, 255, 255, 0, 0, 0, " ☲              ☷ ");
    vec![w.clone(), l2, l3, l4, w.clone(), w]
}

fn flag_singapore() -> Vec<String> {
    let r1 = rgb_bg(237, 41, 57, 255, 255, 255, "  ☾ ★           ") + &rgb_block(237, 41, 57, "");
    let r2 = rgb_bg(237, 41, 57, 255, 255, 255, "    ★★          ") + &rgb_block(237, 41, 57, "");
    let r3 = rgb_block(237, 41, 57, "████████████████");
    let w = rgb_block(255, 255, 255, "████████████████");
    vec![r1, r2, r3, w.clone(), w.clone(), w]
}

fn flag_hong_kong() -> Vec<String> {
    let r = rgb_block(222, 41, 16, "████████████████");
    let l2 = rgb_bg(222, 41, 16, 255, 255, 255, "     ╭🌸╮       ") + &rgb_block(222, 41, 16, "");
    let l3 = rgb_bg(222, 41, 16, 255, 255, 255, "     ╰🌸╯       ") + &rgb_block(222, 41, 16, "");
    vec![r.clone(), l2, l3, r.clone(), r.clone(), r]
}

fn flag_ireland() -> Vec<String> {
    let g = rgb_block(22, 155, 98, "█████");
    let w = rgb_block(255, 255, 255, "██████");
    let o = rgb_block(255, 136, 62, "█████");
    let line = format!("{}{}{}", g, w, o);
    vec![
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line,
    ]
}

fn flag_australia() -> Vec<String> {
    let b = "\x1b[38;2;1;33;105m";
    let w = "\x1b[38;2;255;255;255m";
    let r = "\x1b[38;2;200;16;46m";
    let rst = "\x1b[0m";

    vec![
        format!("{}█{}██{}█{}██{}█{}  ★  ★  {}", r, w, b, w, r, w, rst),
        format!("{}████████{}    ★   {}", r, w, rst),
        format!("{}█{}██{}█{}██{}█{}  ★     {}", b, r, w, w, r, w, rst),
        format!("{}████████████████{}", b, rst),
        format!("{}  ★         ★   {}", w, rst),
        format!("{}████████████████{}", b, rst),
    ]
}

fn flag_austria() -> Vec<String> {
    let r = rgb_block(237, 41, 57, "████████████████");
    let w = rgb_block(255, 255, 255, "████████████████");
    vec![r.clone(), r.clone(), w.clone(), w, r.clone(), r]
}

fn flag_belgium() -> Vec<String> {
    let k = rgb_block(35, 35, 35, "█████");
    let y = rgb_block(253, 218, 36, "██████");
    let r = rgb_block(239, 51, 64, "█████");
    let line = format!("{}{}{}", k, y, r);
    vec![
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line,
    ]
}

fn flag_czech() -> Vec<String> {
    let b1 = rgb_block(17, 69, 126, "██");
    let b2 = rgb_block(17, 69, 126, "████");
    let b3 = rgb_block(17, 69, 126, "██████");
    let w14 = rgb_block(255, 255, 255, "██████████████");
    let w12 = rgb_block(255, 255, 255, "████████████");
    let w10 = rgb_block(255, 255, 255, "██████████");
    let r10 = rgb_block(215, 20, 26, "██████████");
    let r12 = rgb_block(215, 20, 26, "████████████");
    let r14 = rgb_block(215, 20, 26, "██████████████");

    vec![
        format!("{}{}", b1, w14),
        format!("{}{}", b2, w12),
        format!("{}{}", b3, w10),
        format!("{}{}", b3, r10),
        format!("{}{}", b2, r12),
        format!("{}{}", b1, r14),
    ]
}

fn flag_denmark() -> Vec<String> {
    let r4 = rgb_block(200, 16, 46, "████");
    let w2 = rgb_block(255, 255, 255, "██");
    let r10 = rgb_block(200, 16, 46, "██████████");
    let w_full = rgb_block(255, 255, 255, "████████████████");

    let line = format!("{}{}{}", r4, w2, r10);
    vec![
        line.clone(),
        line.clone(),
        w_full.clone(),
        w_full,
        line.clone(),
        line,
    ]
}

fn flag_greece() -> Vec<String> {
    let b = "\x1b[38;2;13;94;175m";
    let w = "\x1b[38;2;255;255;255m";
    let rst = "\x1b[0m";

    vec![
        format!("{}██{}██{}██{}██████████{}", b, w, b, b, rst),
        format!("{}██████{}██████████{}", w, w, rst),
        format!("{}██{}██{}██{}██████████{}", b, w, b, b, rst),
        format!("{}████████████████{}", w, rst),
        format!("{}████████████████{}", b, rst),
        format!("{}████████████████{}", w, rst),
    ]
}

fn flag_hungary() -> Vec<String> {
    let r = rgb_block(206, 41, 57, "████████████████");
    let w = rgb_block(255, 255, 255, "████████████████");
    let g = rgb_block(71, 112, 80, "████████████████");
    vec![r.clone(), r, w.clone(), w, g.clone(), g]
}

fn flag_israel() -> Vec<String> {
    let w = rgb_block(255, 255, 255, "████████████████");
    let b = rgb_block(0, 56, 184, "████████████████");
    let star1 = rgb_bg(255, 255, 255, 0, 56, 184, "     ✡✡✡        ") + &rgb_block(255, 255, 255, "");
    let star2 = rgb_bg(255, 255, 255, 0, 56, 184, "      ✡         ") + &rgb_block(255, 255, 255, "");
    vec![b.clone(), w.clone(), star1, star2, w, b]
}

fn flag_portugal() -> Vec<String> {
    let g = rgb_block(0, 102, 0, "██████");
    let r = rgb_block(255, 0, 0, "██████████");
    let l2 = rgb_bg(0, 102, 0, 255, 215, 0, "    ╭") + &rgb_bg(255, 0, 0, 255, 215, 0, "─╮        ");
    let l3 = rgb_bg(0, 102, 0, 255, 215, 0, "    ╰") + &rgb_bg(255, 0, 0, 255, 215, 0, "─╯        ");
    vec![
        format!("{}{}", g, r),
        l2,
        l3,
        format!("{}{}", g, r),
        format!("{}{}", g, r),
        format!("{}{}", g, r),
    ]
}

fn flag_romania() -> Vec<String> {
    let b = rgb_block(0, 43, 127, "█████");
    let y = rgb_block(252, 209, 22, "██████");
    let r = rgb_block(206, 17, 38, "█████");
    let line = format!("{}{}{}", b, y, r);
    vec![
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line.clone(),
        line,
    ]
}

fn flag_argentina() -> Vec<String> {
    let b = rgb_block(117, 170, 219, "████████████████");
    let w_sun1 = rgb_bg(255, 255, 255, 255, 184, 28, "      ☼☼        ") + &rgb_block(255, 255, 255, "");
    let w_sun2 = rgb_bg(255, 255, 255, 255, 184, 28, "      ☼☼        ") + &rgb_block(255, 255, 255, "");
    vec![b.clone(), b.clone(), w_sun1, w_sun2, b.clone(), b]
}

fn flag_generic(code: &str) -> Vec<String> {
    let c = if code.is_empty() { "??" } else { code };
    let border = rgb_block(59, 130, 246, "████████████████");
    let line2 = rgb_bg(30, 41, 59, 147, 197, 253, "   ╭────────╮   ");
    let line3 = format!(
        "\x1b[48;2;30;41;59m\x1b[38;2;250;204;21m   │   {:^2}   │   \x1b[0m",
        c
    );
    let line4 = rgb_bg(30, 41, 59, 147, 197, 253, "   ╰────────╯   ");
    let line5 = rgb_bg(30, 41, 59, 56, 189, 248, "     🌐🌍🌐     ");

    vec![border.clone(), line2, line3, line4, line5, border]
}
