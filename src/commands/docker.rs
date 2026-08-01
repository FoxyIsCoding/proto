use crate::panel::{PanelPayload, PanelRow};
use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;
use std::process::Command;

#[derive(Subcommand, Debug, Clone)]
pub enum DockerAction {
    #[command(about = "Interactive container manager (default)")]
    Containers {
        #[arg(long, help = "Stream container list to the panel webserver")]
        serve: bool,
        #[arg(long, default_value_t = crate::panel::default_port(), help = "Panel port")]
        port: u16,
    },
    #[command(
        name = "prune-safe",
        about = "Remove dangling images, stopped containers & volumes except those tied to the current project"
    )]
    PruneSafe,
}

pub fn run(action: &DockerAction) {
    if !crate::utils::which("docker") {
        eprintln!("{} docker required.", style::error(""));
        return;
    }
    match action {
        DockerAction::Containers { serve, port } => {
            if *serve {
                serve_mode(*port);
            } else {
                interactive();
            }
        }
        DockerAction::PruneSafe => prune_safe(),
    }
}

struct Container {
    id: String,
    image: String,
    status: String,
    names: String,
}

fn list_containers(all: bool) -> Option<Vec<Container>> {
    let mut cmd = Command::new("docker");
    cmd.args(["ps"]);
    if all {
        cmd.arg("-a");
    }
    let out = cmd
        .args(["--format", "{{.ID}}|{{.Image}}|{{.Status}}|{{.Names}}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut list = Vec::new();
    for line in text.lines() {
        let mut it = line.splitn(4, '|');
        let (Some(id), Some(image), Some(status), Some(names)) = (
            it.next(),
            it.next(),
            it.next(),
            it.next(),
        ) else {
            continue;
        };
        list.push(Container {
            id: id.to_string(),
            image: image.to_string(),
            status: status.to_string(),
            names: names.to_string(),
        });
    }
    Some(list)
}

fn interactive() {
    println!("{}", style::header("Docker Containers"));
    println!("{}", style::divider());

    loop {
        let list = match list_containers(true) {
            Some(l) => l,
            None => {
                eprintln!("{} Could not list containers.", style::error(""));
                return;
            }
        };

        let mut options: Vec<String> = Vec::new();
        options.push("↻ Refresh".to_string());
        for c in &list {
            let status = if c.status.contains("Up") {
                c.status.style(style::Theme::SUCCESS)
            } else if c.status.contains("Exited") {
                c.status.style(style::Theme::MUTED)
            } else {
                c.status.style(style::Theme::WARN)
            };
            options.push(format!(
                "{}  {}  [{}]",
                c.names.style(style::Theme::VALUE),
                c.image.dimmed(),
                status
            ));
        }
        options.push("Done".to_string());

        let idx = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select a container")
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

        let container = &list[idx - 1];
        act_on(container);
    }
}

fn act_on(c: &Container) {
    let actions = vec![
        "start".to_string(),
        "stop".to_string(),
        "restart".to_string(),
        "logs".to_string(),
        "inspect".to_string(),
        "remove".to_string(),
        "back".to_string(),
    ];
    let idx = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("{} ({})", c.names, c.id.chars().take(12).collect::<String>()))
        .items(&actions)
        .default(6)
        .interact()
        .unwrap_or(6);

    match actions[idx].as_str() {
        "start" => run_docker(&["start", &c.id]),
        "stop" => run_docker(&["stop", &c.id]),
        "restart" => run_docker(&["restart", &c.id]),
        "remove" => {
            let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt(format!("Remove container {}?", c.names))
                .default(false)
                .interact()
                .unwrap_or(false);
            if confirm {
                run_docker(&["rm", "-f", &c.id]);
            }
        }
        "logs" => {
            println!();
            let status = Command::new("docker")
                .args(["logs", "-f", "--tail", "40", &c.id])
                .status();
            if let Ok(s) = status {
                let _ = s;
            }
        }
        "inspect" => {
            println!();
            let out = Command::new("docker")
                .args(["inspect", &c.id])
                .output();
            if let Ok(o) = out {
                let text = String::from_utf8_lossy(&o.stdout);
                println!("{}", text);
            }
            pause();
        }
        _ => {}
    }
}

fn run_docker(args: &[&str]) {
    match Command::new("docker").args(args).status() {
        Ok(_) => {}
        Err(e) => eprintln!("  {} {}", style::error(""), e),
    }
}

fn pause() {
    use std::io::Write;
    print!("  {} Press Enter to continue...", "▼".style(style::Theme::MUTED));
    let _ = std::io::stdout().flush();
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}

fn serve_mode(port: u16) {
    let kind = "docker";
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
        "  {} Streaming containers to the panel. Ctrl+C to stop.",
        "◉".style(style::Theme::ACCENT)
    );

    loop {
        let list = list_containers(true).unwrap_or_default();
        let running = list.iter().filter(|c| c.status.contains("Up")).count();
        let mut p = PanelPayload::new("Docker Containers", kind);
        p.metrics = vec![
            crate::panel::PanelMetric::new("Total", &list.len().to_string()),
            crate::panel::PanelMetric::new("Running", &running.to_string()),
        ];
        p.rows = list
            .iter()
            .map(|c| {
                PanelRow::new(&c.names)
                    .desc(&c.image)
                    .cell("ID", &c.id.chars().take(12).collect::<String>())
                    .cell("Status", &c.status)
            })
            .collect();
        let _ = crate::panel::ingest(port, &p);
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}

fn git_branch() -> Option<String> {
    let out = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if b.is_empty() {
        None
    } else {
        Some(b)
    }
}

fn system_df() -> String {
    Command::new("docker")
        .args(["system", "df"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn prune_safe() {
    println!("{}", style::header("Docker Prune (safe)"));
    println!("{}", style::divider());
    println!("  {} Cleaning dangling images, stopped containers, and unused volumes.\n", style::muted(""));

    let branch = git_branch();
    match &branch {
        Some(b) => println!("  {}", style::label_value("Protecting branch", b)),
        None => println!("  {} Not in a git repo — only truly dangling objects will be removed.", style::warn("")),
    }
    println!();
    println!("  {} BEFORE:\n{}", "◉".style(style::Theme::ACCENT), indent_df(&system_df()));

    let images: Vec<String> = docker_ids(&["images", "-q", "-f", "dangling=true"]);
    let containers: Vec<String> = stopped_containers(&branch);
    let volumes: Vec<String> = docker_ids(&["volume", "ls", "-q", "-f", "dangling=true"]);

    let count = images.len() + containers.len() + volumes.len();
    if count == 0 {
        println!("{} Nothing to prune.", style::success(""));
        return;
    }

    println!(
        "\n  {} will remove:\n   • {} dangling image(s)\n   • {} stopped container(s)\n   • {} unused volume(s)",
        style::warn(""),
        images.len(),
        containers.len(),
        volumes.len()
    );

    let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Proceed with prune-safe?")
        .default(false)
        .interact()
        .unwrap_or(false);
    if !confirm {
        println!("{} Aborted.", style::muted(""));
        return;
    }

    let mut removed = 0;
    for id in &images {
        if docker_rm(&["image", "rm", id]) {
            removed += 1;
        }
    }
    for c in &containers {
        if docker_rm(&["rm", "-f", c]) {
            removed += 1;
        }
    }
    for v in &volumes {
        if docker_rm(&["volume", "rm", v]) {
            removed += 1;
        }
    }

    println!();
    println!("  {} AFTER:\n{}", "◉".style(style::Theme::ACCENT), indent_df(&system_df()));
    println!("\n  {} Removed {} object(s).", style::success(""), removed);
}

fn docker_ids(args: &[&str]) -> Vec<String> {
    let out = Command::new("docker").args(args).output();
    match out {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        _ => Vec::new(),
    }
}

fn docker_rm(args: &[&str]) -> bool {
    Command::new("docker")
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn stopped_containers(branch: &Option<String>) -> Vec<String> {
    let out = Command::new("docker")
        .args(["ps", "-a", "-f", "status=exited", "--format", "{{.ID}}|{{.Image}}|{{.Names}}"])
        .output();
    let text = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };
    let mut keep = Vec::new();
    for line in text.lines() {
        let mut it = line.splitn(3, '|');
        let (Some(id), Some(image), Some(names)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let protected = match branch {
            Some(b) => names.contains(b) || image.contains(b),
            None => false,
        };
        if !protected {
            keep.push(id.to_string());
        }
    }
    keep
}

fn indent_df(df: &str) -> String {
    df.lines()
        .map(|l| format!("    {}", l))
        .collect::<Vec<_>>()
        .join("\n")
}
