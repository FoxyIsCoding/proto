use crate::style;
use owo_colors::OwoColorize;

const RECORD_TYPES: &[&str] = &["A", "AAAA", "CNAME", "MX", "TXT", "NS"];

pub fn run(domain: &str) {
    if !crate::utils::which("dig") {
        eprintln!(
            "{} dig required. Install bind-tools (e.g. {}).",
            style::error(""),
            "sudo pacman -S bind-tools".dimmed()
        );
        return;
    }

    println!(
        "{}",
        style::header(format!("DNS Lookup: {}", domain).as_str())
    );
    println!("{}", style::divider());

    let mut found = false;
    for rt in RECORD_TYPES {
        let out =
            crate::utils::run_command_output("dig", &["+short", domain, rt]).unwrap_or_default();
        let values: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();

        if values.is_empty() {
            continue;
        }
        found = true;

        let label = format!("{:5}", rt);
        for (i, v) in values.iter().enumerate() {
            if i == 0 {
                println!("  {} {}", label.style(style::Theme::ACCENT).bold(), v);
            } else {
                println!("  {} {}", " ".repeat(5), v);
            }
        }
    }

    println!();
    if found {
        println!(
            "  {} All records above resolved for {}",
            style::success(""),
            domain
        );
    } else {
        println!(
            "  {} No records found — domain may not exist or has no records.",
            style::warn("")
        );
    }
}
