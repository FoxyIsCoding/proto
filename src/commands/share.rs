use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum ShareAction {
    #[command(about = "Create a shareable terminal session")]
    Create,
    #[command(about = "Join an existing shared session")]
    Join {
        #[arg(required = true, value_name = "LINK")]
        link: String,
    },
}

pub fn run(action: &ShareAction) {
    match action {
        ShareAction::Create => create(),
        ShareAction::Join { link } => join(link),
    }
}

fn create() {
    println!("{} {}", "◆".style(style::Theme::ACCENT), "Share Session".style(style::Theme::HEADER));
    println!("{}", style::divider());

    let has_tmate = crate::utils::which("tmate");
    let has_tmux = crate::utils::which("tmux");

    if has_tmate {
        share_with_tmate();
    } else if has_tmux {
        share_with_tmux();
    } else {
        eprintln!("\n{} Neither tmate nor tmux found.", style::error(""));
        println!("\n{}", "Install one of:".style(style::Theme::HEADER));
        println!("  {} {}", "▸".style(style::Theme::ACCENT), "tmate  — remote pair programming (recommended)");
        println!("    {}", "sudo apt install tmate  /  sudo pacman -S tmate".dimmed());
        println!("  {} {}", "▸".style(style::Theme::ACCENT), "tmux   — local terminal sharing");
        println!("    {}", "sudo apt install tmux  /  sudo pacman -S tmux".dimmed());
    }
}

fn share_with_tmate() {
    let socket = format!("/tmp/proto-tmate-{}", std::process::id());
    let sp = style::Spinner::new("Starting tmate session...");

    let sock_path = std::path::Path::new(&socket);
    if sock_path.exists() { let _ = std::fs::remove_file(sock_path); }

    let start = std::process::Command::new("tmate")
        .args(["-S", &socket, "new-session", "-d"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if start.map(|s| !s.success()).unwrap_or(true) {
        sp.fail("Failed to start tmate session.");
        return;
    }

    sp.update("Waiting for tmate to connect...");

    let ready = std::process::Command::new("tmate")
        .args(["-S", &socket, "wait", "tmate-ready"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if ready.map(|s| !s.success()).unwrap_or(true) {
        sp.fail("tmate failed to establish connection.");
        let _ = std::process::Command::new("tmate").args(["-S", &socket, "kill-session"]).status();
        return;
    }

    let ssh = crate::utils::run_command_output("tmate", &["-S", &socket, "display", "-p", "#{tmate_ssh}"])
        .unwrap_or_default();
    let web = crate::utils::run_command_output("tmate", &["-S", &socket, "display", "-p", "#{tmate_web}"])
        .unwrap_or_default();

    sp.done("Session ready");

    println!();
    println!("{}", "Share Links".style(style::Theme::HEADER));
    println!("{}", style::divider());
    println!("{}", style::label_value("SSH", &ssh));
    if !web.is_empty() && web != ssh {
        println!("{}", style::label_value("Web", &web));
    }
    println!("{}", style::divider());

    println!("\n{}", "Commands:".style(style::Theme::HEADER));
    println!("  {}     {}", "proto share-session join <link>".style(style::Theme::ACCENT), "join from another machine".dimmed());
    println!("  {}               {}", "proto share-session create".style(style::Theme::ACCENT), "re-share this session".dimmed());
    println!("  {}               {}", "/quit".style(style::Theme::ACCENT), "end the session".dimmed());

    println!("\n{} Attaching to session (Ctrl+B D to detach)...\n", "  ".dimmed());
    println!("{}", style::divider());

    let status = std::process::Command::new("tmate")
        .args(["-S", &socket, "attach"])
        .status();

    let _ = std::process::Command::new("tmate").args(["-S", &socket, "kill-session"]).status();
    if status.is_err() { return; }

    println!("\n{} Session ended.", style::success(""));
}

fn share_with_tmux() {
    let sp = style::Spinner::new("Setting up tmux session...");

    let in_tmux = std::env::var("TMUX").is_ok();

    if in_tmux {
        let socket = std::env::var("TMUX").unwrap_or_default();
        let session = crate::utils::run_command_output("tmux", &["display-message", "-p", "#S"])
            .unwrap_or_else(|_| "proto".into());

        sp.done("Using current tmux session");

        println!();
        println!("{}", style::label_value("Session", &session));
        println!("{}", style::label_value("Socket", &socket));

        let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
        let host = crate::utils::run_command_output("hostname", &[]).unwrap_or_else(|_| "localhost".into());

        println!("\n{} Share locally:", style::Theme::HEADER.style(" ".to_string()));
        println!("  {} {} {}", "▸".style(style::Theme::ACCENT), "tmux attach -t".dimmed(), session.style(style::Theme::ACCENT));
        println!("\n  {} Have your teammate SSH in, then run the attach command above.", "  ".dimmed());
        let ssh_cmd = format!("ssh {}@{}", user, host);
        println!("  {}", ssh_cmd.dimmed());
    } else {
        let session_name = format!("proto-{}", std::process::id());

        let start = std::process::Command::new("tmux")
            .args(["new-session", "-d", "-s", &session_name])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match start {
            Ok(s) if s.success() => {
                sp.done(&format!("Session '{}' created", session_name));

                let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
                let host = crate::utils::run_command_output("hostname", &[]).unwrap_or_else(|_| "localhost".into());

                println!();
                println!("{}", "Share Session".style(style::Theme::HEADER));
                println!("{}", style::divider());
                println!("{}", style::label_value("Session", &session_name));
                println!("{}", style::label_value("Command", &format!("tmux attach -t {}", session_name)));
                println!("{}", style::divider());
                println!("\n  {} To share: your teammate SSHs in and runs:", "  ".dimmed());
                println!("  {}", format!("ssh {}@{}", user, host).style(style::Theme::ACCENT));
                println!("  {}", format!("tmux attach -t {}", session_name).style(style::Theme::ACCENT));

                println!("\n{} Attaching to session (Ctrl+B D to detach)...\n", "  ".dimmed());
                println!("{}", style::divider());

                let _ = std::process::Command::new("tmux")
                    .args(["attach", "-t", &session_name])
                    .status();

                println!("\n{} Session detached (still running: {}).", style::success(""), format!("tmux attach -t {}", session_name).style(style::Theme::ACCENT));
            }
            _ => {
                sp.fail("Failed to create tmux session. Is tmux installed?");
            }
        }
    }
}

fn join(link: &str) {
    let sp = style::Spinner::new(&format!("Joining {}...", link));

    if link.starts_with("ssh ") {
        let addr = link.trim_start_matches("ssh ").trim();
        sp.done(&format!("Connecting to {}", addr));
        println!("\n{} Connecting via SSH...", "  ".dimmed());
        let _ = std::process::Command::new("ssh").arg(addr).status();
    } else if link.contains("tmate.io") || link.starts_with("http") {
        sp.done(&format!("Opening {}", link));
        println!("\n{} Open in browser: {}", "  ".dimmed(), link.style(style::Theme::ACCENT));
        if crate::utils::which("xdg-open") {
            let _ = std::process::Command::new("xdg-open").arg(link).status();
        }
    } else {
        sp.fail("Unrecognized link format. Use tmate SSH or web link.");
    }
}
