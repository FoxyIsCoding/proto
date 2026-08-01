use crate::panel::{PanelPayload, PanelRow};
use crate::style;
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::process::Command;

struct CacheTarget {
    label: &'static str,
    path: Option<PathBuf>,
    needs_sudo: bool,
    cmd: Option<(&'static str, Vec<String>)>,
}

pub fn run(serve: bool, port: u16) {
    if serve {
        serve_scan(port);
        return;
    }
    interactive();
}

fn cache_targets() -> Vec<CacheTarget> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    vec![
        CacheTarget {
            label: "npm cache",
            path: Some(home.join(".npm/_cacache")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "pip cache",
            path: Some(home.join(".cache/pip")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "uv cache",
            path: Some(home.join(".cache/uv")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "bun cache",
            path: Some(home.join(".bun/install/cache")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "yarn cache",
            path: Some(home.join(".cache/yarn")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "cargo registry",
            path: Some(home.join(".cargo/registry/cache")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "yay cache",
            path: Some(home.join(".cache/yay")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "paru cache",
            path: Some(home.join(".cache/paru")),
            needs_sudo: false,
            cmd: None,
        },
        CacheTarget {
            label: "pacman pkg cache",
            path: Some(PathBuf::from("/var/cache/pacman/pkg")),
            needs_sudo: true,
            cmd: None,
        },
        CacheTarget {
            label: "apt archives",
            path: Some(PathBuf::from("/var/cache/apt/archives")),
            needs_sudo: true,
            cmd: None,
        },
        CacheTarget {
            label: "dnf cache",
            path: Some(PathBuf::from("/var/cache/dnf")),
            needs_sudo: true,
            cmd: None,
        },
        CacheTarget {
            label: "docker builder cache",
            path: None,
            needs_sudo: false,
            cmd: Some(("docker", vec!["builder".into(), "prune".into(), "-f".into()])),
        },
    ]
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let out = Command::new("du")
        .args(["-sb", path.to_str().unwrap_or("")])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            text.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0)
        }
        _ => 0,
    }
}

fn docker_builder_size() -> u64 {
    let out = Command::new("docker")
        .args(["system", "df", "--format", "{{.Size}}"])
        .output();
    let text = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return 0,
    };
    // Rows: Images, Containers, Local Volumes, Build Cache (last).
    text.lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        })
        .next_back()
        .map(parse_docker_size)
        .unwrap_or(0)
}

fn parse_docker_size(s: &str) -> u64 {
    // Convert a human size like "1.234GB" to bytes (approximation).
    let s = s.trim();
    let (num, mult) = if let Some(v) = s.strip_suffix("GB") {
        (v, 1_000_000_000u64)
    } else if let Some(v) = s.strip_suffix("MB") {
        (v, 1_000_000)
    } else if let Some(v) = s.strip_suffix("KB") {
        (v, 1_000)
    } else if let Some(v) = s.strip_suffix("B") {
        (v, 1)
    } else {
        (s, 1)
    };
    num.trim().parse::<f64>().unwrap_or(0.0) as u64 * mult
}

struct ScanEntry {
    target: CacheTarget,
    size: u64,
}

fn scan() -> Vec<ScanEntry> {
    cache_targets()
        .into_iter()
        .map(|target| {
            let size = if target.cmd.is_some() {
                docker_builder_size()
            } else if let Some(p) = &target.path {
                dir_size(p)
            } else {
                0
            };
            ScanEntry { target, size }
        })
        .collect()
}

fn disk_free() -> u64 {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let home = home.to_string_lossy().to_string();
    let out = Command::new("df")
        .args(["-B1", "--output=avail", &home])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next_back()
                .and_then(|l| l.trim().parse().ok())
                .unwrap_or(0)
        }
        _ => 0,
    }
}

fn interactive() {
    println!("{}", style::header("Cache Cleaner"));
    println!("{}", style::divider());

    let entries = scan();
    let total: u64 = entries.iter().map(|e| e.size).sum();
    let free_before = disk_free();

    println!();
    println!("  {}", style::label_value("Disk free (before)", &crate::utils::format_size(free_before)));
    println!("  {}", style::label_value("Total cache", &crate::utils::format_size(total)));
    println!();
    println!(
        "  {:>18}  {:<24} {}",
        "SIZE".style(style::Theme::LABEL),
        "CACHE".style(style::Theme::LABEL),
        "PATH".style(style::Theme::LABEL),
    );

    let mut options: Vec<String> = Vec::new();
    let mut non_empty: Vec<usize> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        if e.size == 0 {
            continue;
        }
        non_empty.push(i);
        let path = e
            .target
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "docker builder".into());
        let sudo = if e.target.needs_sudo { " (sudo)" } else { "" };
        println!(
            "  {:>18}  {:<24} {}",
            crate::utils::format_size(e.size).style(style::Theme::VALUE),
            format!("{}{}", e.target.label, sudo).dimmed(),
            path.dimmed()
        );
        options.push(format!(
            "{}  {}",
            e.target.label,
            crate::utils::format_size(e.size)
        ));
    }
    println!();

    if non_empty.is_empty() {
        println!("{} No caches found to clean.", style::success(""));
        return;
    }

    let selected = dialoguer::MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select caches to clean (space to toggle)")
        .items(&options)
        .interact()
        .unwrap_or_default();

    if selected.is_empty() {
        println!("{} Nothing selected.", style::muted(""));
        return;
    }

    let chosen: Vec<&ScanEntry> = selected.iter().map(|&i| &entries[non_empty[i]]).collect();
    let reclaim: u64 = chosen.iter().map(|e| e.size).sum();
    println!();
    for c in &chosen {
        println!(
            "  {} {}  ({} → {})",
            style::warn(""),
            c.target.label,
            crate::utils::format_size(c.size),
            "will be freed".dimmed()
        );
    }
    println!(
        "\n  {}",
        style::label_value("Total", &crate::utils::format_size(reclaim)),
    );

    let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Proceed with cleanup?")
        .default(false)
        .interact()
        .unwrap_or(false);
    if !confirm {
        println!("{} Aborted.", style::muted(""));
        return;
    }

    for c in &chosen {
        let ok = clean_target(&c.target);
        if ok {
            println!("  {} Cleaned {}", style::success(""), c.target.label);
        } else {
            println!("  {} Failed to clean {}", style::error(""), c.target.label);
        }
    }

    let free_after = disk_free();
    println!();
    println!("{}", style::divider());
    println!(
        "  {}",
        style::label_value("Disk free (before)", &crate::utils::format_size(free_before)),
    );
    println!(
        "  {}",
        style::label_value("Disk free (after)", &crate::utils::format_size(free_after)),
    );
    println!(
        "  {}",
        style::label_value("Recovered", &crate::utils::format_size(free_after.saturating_sub(free_before))),
    );
}

fn clean_target(target: &CacheTarget) -> bool {
    if let Some((cmd, args)) = &target.cmd {
        return Command::new(cmd)
            .args(args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    }
    let path = match &target.path {
        Some(p) => p,
        None => return false,
    };
    let rm = "rm";
    let mut full: Vec<String> = Vec::new();
    if target.needs_sudo {
        full.push("sudo".to_string());
    }
    full.push(rm.to_string());
    full.push("-rf".to_string());
    full.push(path.to_string_lossy().to_string());
    let status = Command::new(&full[0])
        .args(&full[1..])
        .status();
    status.map(|s| s.success()).unwrap_or(false)
}

fn serve_scan(port: u16) {
    let kind = "clean-cache";
    if let Err(e) = crate::panel::start(port) {
        eprintln!("{} {}", style::error(""), e);
        return;
    }
    let entries = scan();
    let total: u64 = entries.iter().map(|e| e.size).sum();
    let mut p = PanelPayload::new("Cache Scan", kind);
    p.updated = Some(crate::utils::get_uptime());
    p.metrics = vec![
        crate::panel::PanelMetric::new("Total cache", &crate::utils::format_size(total)),
        crate::panel::PanelMetric::new("Disk free", &crate::utils::format_size(disk_free())),
    ];
    p.rows = entries
        .iter()
        .filter(|e| e.size > 0)
        .map(|e| {
            let mut r = PanelRow::new(e.target.label);
            if let Some(path) = &e.target.path {
                r = r.desc(&path.display().to_string());
            }
            r = r.cell("size", &crate::utils::format_size(e.size));
            if e.target.needs_sudo {
                r = r.cell("access", "sudo");
            }
            r
        })
        .collect();
    let _ = crate::panel::ingest(port, &p);
    println!(
        "  {}",
        style::label_value("Panel", &crate::panel::panel_url(port, kind)),
    );
    crate::panel::open(port, kind);
    println!("  {} Cache scan sent to the panel.", style::success(""));
}
