use crate::style;
use owo_colors::OwoColorize;
use std::process::Command;

struct HeavyProc {
    pid: u32,
    name: String,
    cpu: f64,
    mem: f64,
    rss_kb: u64,
}

pub fn run(min_cpu: f64, min_mem_mb: u64, all: bool, serve: bool, port: u16) {
    if serve {
        serve_scan(min_cpu, min_mem_mb, all, port);
        return;
    }
    println!("{}", style::header("Heavy Process Scanner"));
    println!("{}", style::divider());

    let procs = match list_procs() {
        Some(p) => p,
        None => {
            eprintln!("{} Could not read process list (ps required).", style::error(""));
            return;
        }
    };

    let self_pid = std::process::id();
    let heavy: Vec<HeavyProc> = procs
        .into_iter()
        .filter(|p| p.pid >= 100 && p.pid != self_pid)
        .filter(|p| {
            all || p.cpu >= min_cpu || (p.rss_kb as f64 / 1024.0) >= min_mem_mb as f64
        })
        .collect();

    if heavy.is_empty() {
        println!("{} No heavy processes found.", style::success(""));
        return;
    }

    let shown = heavy.len().min(15);
    println!(
        "  {} Top heavy processes (CPU >{}% or RAM >{}MB):\n",
        style::warn(""),
        min_cpu,
        min_mem_mb
    );

    use dialoguer::{Confirm, MultiSelect};
    let items: Vec<String> = heavy[..shown]
        .iter()
        .map(|p| {
            format!(
                "{}  {}  {}%  {:.1}%  {}",
                p.pid.to_string().style(style::Theme::ACCENT).bold(),
                p.name.style(style::Theme::VALUE),
                format!("{:>4}", p.cpu).dimmed(),
                p.mem,
                crate::utils::format_size(p.rss_kb * 1024).dimmed(),
            )
        })
        .collect();

    let selected = MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select processes to kill (space to toggle, enter to confirm)")
        .items(&items)
        .interact()
        .unwrap_or_default();

    if selected.is_empty() {
        println!("{} Nothing selected.", style::muted(""));
        return;
    }

    let targets: Vec<&HeavyProc> = selected.iter().map(|&i| &heavy[i]).collect();
    println!();
    for t in &targets {
        println!("  {} {} (pid {})", "✗".style(style::Theme::WARN), t.name, t.pid);
    }

    let proceed = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Kill these processes?")
        .default(false)
        .interact()
        .unwrap_or(false);
    if !proceed {
        println!("{} Aborted.", style::muted(""));
        return;
    }

    let mut killed = 0;
    for t in targets {
        match kill_proc(t.pid) {
            true => {
                println!("  {} Killed {} (pid {})", style::success(""), t.name, t.pid);
                killed += 1;
            }
            false => println!(
                "  {} Failed to kill {} (pid {})",
                style::error(""),
                t.name,
                t.pid
            ),
        }
    }
    println!(
        "\n  {} {} process(es) killed.",
        style::success(""),
        killed
    );
}

fn serve_scan(min_cpu: f64, min_mem_mb: u64, all: bool, port: u16) {
    let kind = "kill-heavy";
    if let Err(e) = crate::panel::start(port) {
        eprintln!("{} {}", style::error(""), e);
        return;
    }
    let procs = list_procs().unwrap_or_default();
    let self_pid = std::process::id();
    let heavy: Vec<HeavyProc> = procs
        .into_iter()
        .filter(|p| p.pid >= 100 && p.pid != self_pid)
        .filter(|p| all || p.cpu >= min_cpu || (p.rss_kb as f64 / 1024.0) >= min_mem_mb as f64)
        .collect();

    let mut p = crate::panel::PanelPayload::new("Heavy Processes", kind);
    p.updated = Some(crate::utils::get_uptime());
    p.metrics = vec![
        crate::panel::PanelMetric::new("Heavy processes", &heavy.len().to_string())
            .status(if heavy.is_empty() { "ok" } else { "warn" }),
    ];
    p.rows = heavy
        .iter()
        .take(30)
        .map(|h| {
            crate::panel::PanelRow::new(&h.name)
                .cell("PID", &h.pid.to_string())
                .cell("CPU", &format!("{:.1}%", h.cpu))
                .cell("MEM", &format!("{:.1}%", h.mem))
                .cell("RSS", &crate::utils::format_size(h.rss_kb * 1024))
        })
        .collect();
    let _ = crate::panel::ingest(port, &p);
    println!(
        "  {}",
        style::label_value("Panel", &crate::panel::panel_url(port, kind)),
    );
    crate::panel::open(port, kind);
    println!("  {} Heavy process scan sent to the panel.", style::success(""));
}

fn list_procs() -> Option<Vec<HeavyProc>> {    let out = Command::new("ps")
        .args([
            "-e",
            "--no-headers",
            "-o",
            "pid,ppid,comm,pcpu,pmem,rss",
            "--sort=-pcpu",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut procs = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let pid: u32 = it.next()?.parse().ok()?;
        let _ppid: u32 = it.next()?.parse().ok()?;
        let name = it.next()?.to_string();
        let cpu: f64 = it.next()?.parse().ok()?;
        let mem: f64 = it.next()?.parse().ok()?;
        let rss_kb: u64 = it.next()?.parse().ok()?;
        procs.push(HeavyProc {
            pid,
            name,
            cpu,
            mem,
            rss_kb,
        });
    }
    Some(procs)
}

fn kill_proc(pid: u32) -> bool {
    if !Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(800));
    if std::path::Path::new(&format!("/proc/{}/", pid)).exists() {
        Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        true
    }
}
