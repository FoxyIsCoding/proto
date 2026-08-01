use crate::panel::{PanelPayload, PanelRow};
use crate::style;
use owo_colors::OwoColorize;
use std::process::Command;

struct PortInfo {
    proto: String,
    local: String,
    port: u16,
    pid: Option<u32>,
    process: Option<String>,
}

pub fn run(serve: bool, port: u16) {
    if serve {
        serve_mode(port);
        return;
    }
    interactive();
}

fn serve_mode(port: u16) {
    let kind = "ports";
    if let Err(e) = crate::panel::start(port) {
        eprintln!("{} {}", style::error(""), e);
    } else {
        println!(
            "  {}",
            style::label_value("Panel", &crate::panel::panel_url(port, kind)),
        );
        crate::panel::open(port, kind);
    }
    println!();
    println!(
        "  {} Streaming listening ports to the panel. Ctrl+C to stop.",
        "◉".style(style::Theme::ACCENT)
    );

    loop {
        let list = scan().unwrap_or_default();
        let mut p = PanelPayload::new("Listening Ports", kind);
        p.updated = Some(crate::utils::get_uptime());
        let tcp = list.iter().filter(|x| x.proto == "tcp").count();
        let udp = list.iter().filter(|x| x.proto == "udp").count();
        p.metrics = vec![
            crate::panel::PanelMetric::new("Listening", &list.len().to_string()),
            crate::panel::PanelMetric::new("TCP", &tcp.to_string()),
            crate::panel::PanelMetric::new("UDP", &udp.to_string()),
        ];
        p.rows = list
            .iter()
            .map(|s| {
                let title = format!("{}:{}", s.proto, s.port);
            let mut r = PanelRow::new(&title);
                if let Some(proc) = &s.process {
                    r = r.desc(proc);
                }
                r = r.cell(
                    "PID",
                    &s.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                )
                .cell("Address", &s.local);
                r
            })
            .collect();
        let _ = crate::panel::ingest(port, &p);

        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}

fn interactive() {
    println!("{}", style::header("Listening Ports"));
    println!("{}", style::divider());

    loop {
        let list = match scan() {
            Some(l) if !l.is_empty() => l,
            Some(_) => {
                println!("{} No listening ports found.", style::muted(""));
                return;
            }
            None => {
                eprintln!("{} Could not read ports (ss required).", style::error(""));
                return;
            }
        };

        print_table(&list);

        let mut options: Vec<String> = Vec::new();
        options.push("↻ Refresh".to_string());
        for s in &list {
            let proc = s.process.clone().unwrap_or_else(|| "?".to_string());
            options.push(format!(
                "{} {}:{}  {}",
                "✗".style(style::Theme::WARN),
                s.port,
                proc,
                s.local.dimmed()
            ));
        }
        options.push("Done".to_string());

        let idx = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select a port to kill (or refresh)")
            .items(&options)
            .default(0)
            .interact()
            .unwrap_or(options.len() - 1);

        if idx == 0 {
            continue;
        }
        if idx >= options.len() - 1 {
            break;
        }

        let target = &list[idx - 1];
        let proc = target.process.clone().unwrap_or_else(|| "unknown".into());
        let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(format!("Kill {} on port {} (pid {})?", proc, target.port, target.pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into())))
            .default(false)
            .interact()
            .unwrap_or(false);
        if !confirm {
            continue;
        }

        if let Some(pid) = target.pid {
            let ok = Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                println!("  {} Killed {} (pid {})\n", style::success(""), proc, pid);
            } else {
                println!("  {} Failed to kill {}\n", style::error(""), proc);
            }
        } else {
            println!("  {} No PID found for that socket.\n", style::warn(""));
        }
    }
}

fn print_table(list: &[PortInfo]) {
    println!(
        "  {} {:>8}  {:6}  {:<24} {}",
        "PROTO".style(style::Theme::LABEL),
        "PORT".style(style::Theme::LABEL),
        "PID".style(style::Theme::LABEL),
        "PROCESS".style(style::Theme::LABEL),
        "ADDRESS".style(style::Theme::LABEL),
    );
    for s in list {
        println!(
            "  {} {:>8}  {:6}  {:<24} {}",
            s.proto.style(style::Theme::ACCENT),
            s.port.to_string().style(style::Theme::VALUE),
            s.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
            s.process.as_deref().unwrap_or("?"),
            s.local.dimmed(),
        );
    }
    println!();
}

fn scan() -> Option<Vec<PortInfo>> {
    let out = Command::new("ss").args(["-tulpnH"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut list = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let proto = it.next()?.to_string();
        let state = it.next()?.to_string();
        let _ = it.next();
        let _ = it.next();
        let local = it.next()?.to_string();
        let _peer = it.next();
        let proc_col = it.next().map(|s| s.to_string());

        if state != "LISTEN" && state != "UNCONN" {
            continue;
        }
        let port: u16 = match local.rsplit(':').next().and_then(|p| p.parse().ok()) {
            Some(p) => p,
            None => continue,
        };
        let (pid, process) = parse_proc_col(proc_col.as_deref());
        list.push(PortInfo {
            proto,
            local,
            port,
            pid,
            process,
        });
    }
    list.sort_by_key(|s| (s.port, s.proto.clone()));
    Some(list)
}

fn parse_proc_col(col: Option<&str>) -> (Option<u32>, Option<String>) {
    let col = match col {
        Some(c) => c,
        None => return (None, None),
    };
    let mut pid = None;
    let mut name = None;
    if let Some(start) = col.find("pid=") {
        let rest = &col[start + 4..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        pid = digits.parse().ok();
    }
    if let Some(start) = col.find("(\"") {
        let rest = &col[start + 2..];
        name = rest.split('"').next().map(|s| s.to_string());
    }
    (pid, name)
}
