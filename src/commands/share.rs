use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum ShareAction {
    #[command(about = "Create a shareable terminal session")]
    Create {
        #[arg(long, value_name = "BACKEND", help = "Force backend: sshx, tmate, tmux, vnc")]
        backend: Option<String>,
    },
    #[command(about = "Join an existing shared session")]
    Join {
        #[arg(required = true, value_name = "LINK")]
        link: String,
    },
}

pub fn run(action: &ShareAction) {
    match action {
        ShareAction::Create { backend } => create(backend.as_deref()),
        ShareAction::Join { link } => join(link),
    }
}

fn create(force_backend: Option<&str>) {
    println!("{} {}", "◆".style(style::Theme::ACCENT), "Share Session".style(style::Theme::HEADER));
    println!("{}", style::divider());

    let backend = force_backend.unwrap_or("auto");

    match backend {
        "sshx" => share_with_sshx(),
        "tmate" => share_with_tmate(true),
        "tmux" => share_with_tmux(),
        "vnc" => share_with_vnc(),
        _ => {
            if crate::utils::which("sshx") {
                share_with_sshx();
            } else if crate::utils::which("tmate") {
                share_with_tmate(false);
            } else if crate::utils::which("tmux") {
                share_with_tmux();
            } else {
                eprintln!("\n{} No sharing backend found.", style::error(""));
                println!("\n{}", "Install one of:".style(style::Theme::HEADER));
                println!("  {} {}", "▸".style(style::Theme::ACCENT), "sshx  — web link, viewer opens in browser (teamviewer-style)");
                println!("    {}", "cargo install sshx".dimmed());
                println!("  {} {}", "▸".style(style::Theme::ACCENT), "tmate — SSH + web link, needs tmate.io relay");
                println!("    {}", "sudo pacman -S tmate  /  sudo apt install tmate".dimmed());
                println!("  {} {}", "▸".style(style::Theme::ACCENT), "vnc   — full desktop (entire screen), VNC + ngrok tunnel");
                println!("    {}", "sudo pacman -S x11vnc wayvnc ngrok".dimmed());
                println!("  {} {}", "▸".style(style::Theme::ACCENT), "tmux  — local only, teammate must SSH in");
                println!("    {}", "sudo pacman -S tmux  /  sudo apt install tmux".dimmed());
                println!("\n{} {}", "Or force a backend:", "proto share-session create --backend tmux".dimmed());
            }
        }
    }
}

fn share_with_sshx() {
    if !crate::utils::which("sshx") {
        eprintln!("{} sshx not installed. Run: {}", style::error(""), "cargo install sshx".style(style::Theme::ACCENT));
        return;
    }

    println!("\n{} Starting sshx session...", "  ".dimmed());
    println!("{} Share the link below. Viewer opens in their browser — no install needed.", "  ".dimmed());
    println!("{} Ctrl+C to end the session.\n", "  ".dimmed());
    println!("{}", style::divider());

    let status = std::process::Command::new("sshx")
        .status();

    match status {
        Ok(s) if s.success() => println!("\n{} Session ended.", style::success("")),
        _ => eprintln!("\n{} sshx session terminated.", style::warn("")),
    }
}

fn share_with_tmate(forced: bool) {
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
        fallback_tmux("tmate");
        return;
    }

    sp.update("Waiting for tmate relay (may take up to 30s)...");

    let ready = std::process::Command::new("timeout")
        .args(["30", "tmate", "-S", &socket, "wait", "tmate-ready"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if ready.map(|s| !s.success()).unwrap_or(true) {
        sp.fail("tmate relay unreachable (tmate.io may be down).");
        let _ = std::process::Command::new("tmate").args(["-S", &socket, "kill-session"]).status();

        println!("{} tmate relay returned 503/connection error.", style::warn(""));
        if !forced {
            fallback_tmux("tmate");
        }
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

    println!("\n{} Attaching (Ctrl+B D to detach)...\n", "  ".dimmed());
    println!("{}", style::divider());

    let status = std::process::Command::new("tmate")
        .args(["-S", &socket, "attach"])
        .status();

    let _ = std::process::Command::new("tmate").args(["-S", &socket, "kill-session"]).status();
    if status.is_err() { return; }

    println!("\n{} Session ended.", style::success(""));
}

fn fallback_tmux(from: &str) {
    if crate::utils::which("tmux") {
        println!("\n{} Falling back to tmux...", "  ".dimmed());
        share_with_tmux();
    } else {
        println!("{} Install tmux for local session sharing.", style::warn(""));
        println!("{} Or: {}", "  ".dimmed(), "proto share-session create --backend tmux".style(style::Theme::MUTED));
    }
}

fn share_with_tmux() {
    if !crate::utils::which("tmux") {
        eprintln!("{} tmux not installed.", style::error(""));
        return;
    }

    let sp = style::Spinner::new("Setting up tmux session...");

    let in_tmux = std::env::var("TMUX").is_ok();

    if in_tmux {
        let session = crate::utils::run_command_output("tmux", &["display-message", "-p", "#S"])
            .unwrap_or_else(|_| "proto".into());

        sp.done("Using current tmux session");

        println!();
        println!("{}", style::label_value("Session", &session));

        let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
        let host = crate::utils::run_command_output("hostname", &[]).unwrap_or_else(|_| "localhost".into());

        println!("\n{} Share locally:", "  ".dimmed());
        println!("  {} {} {}", "▸".style(style::Theme::ACCENT), "tmux attach -t".dimmed(), session.style(style::Theme::ACCENT));
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
                println!("\n  {} To share: teammate SSHs in and runs:", "  ".dimmed());
                println!("  {}", format!("ssh {}@{}", user, host).style(style::Theme::ACCENT));
                println!("  {}", format!("tmux attach -t {}", session_name).style(style::Theme::ACCENT));

                println!("\n{} Attaching (Ctrl+B D to detach)...\n", "  ".dimmed());
                println!("{}", style::divider());

                let _ = std::process::Command::new("tmux")
                    .args(["attach", "-t", &session_name])
                    .status();

                println!("\n{} Session detached (still running: {}).", style::success(""), format!("tmux attach -t {}", session_name).style(style::Theme::ACCENT));
            }
            _ => {
                sp.fail("Failed to create tmux session.");
            }
        }
    }
}

fn share_with_vnc() {
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("XDG_SESSION_TYPE").map(|s| s == "wayland").unwrap_or(false);

    let (server_bin, install_hint) = if is_wayland {
        ("wayvnc", "sudo pacman -S wayvnc")
    } else {
        ("x11vnc", "sudo pacman -S x11vnc")
    };

    if !crate::utils::which(server_bin) {
        eprintln!("{} {} not installed. Install: {}", style::error(""), server_bin, install_hint.style(style::Theme::ACCENT));
        return;
    }

    let vnc_port = 5900u16;
    let web_port = 5800u16;
    let sp = style::Spinner::new(&format!("Starting {} on port {}...", server_bin, vnc_port));

    let mut child = if is_wayland {
        std::process::Command::new("wayvnc")
            .args(["0.0.0.0", &vnc_port.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
    } else {
        std::process::Command::new("x11vnc")
            .args(["-forever", "-shared", "-rfbport", &vnc_port.to_string(), "-passwd", "proto"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
    };

    let mut child = match child {
        Ok(c) => c,
        Err(e) => { sp.fail(&format!("Failed: {}", e)); return; }
    };

    std::thread::sleep(std::time::Duration::from_secs(1));

    if let Ok(Some(_)) = child.try_wait() {
        sp.fail(&format!("{} exited immediately. Is your display accessible?", server_bin));
        return;
    }

    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".into());
    let pass = "proto";
    sp.done("VNC server running");

    let mut public_url = String::new();
    let mut ngrok_pid: Option<u32> = None;

    if crate::utils::which("ngrok") {
        let sp2 = style::Spinner::new("Starting ngrok tunnel...");
        let result = std::process::Command::new("ngrok")
            .args(["tcp", &vnc_port.to_string(), "--log", "stdout"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(mut t) => {
                ngrok_pid = Some(t.id());
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(t.stdout.take().unwrap());
                let start = std::time::Instant::now();

                for line in reader.lines().flatten() {
                    if let Some(pos) = line.find("url=") {
                        let rest = &line[pos + 4..];
                        public_url = rest.split_whitespace().next().unwrap_or("").to_string();
                        if !public_url.is_empty() { break; }
                    }
                    if start.elapsed() > std::time::Duration::from_secs(30) { break; }
                }

                if !public_url.is_empty() {
                    public_url = public_url.trim_start_matches("tcp://").to_string();
                    sp2.done("Tunnel ready");
                } else {
                    sp2.fail("ngrok timed out after 30s");
                    let _ = t.kill();
                    ngrok_pid = None;
                }
            }
            Err(_) => { sp2.fail("Failed to start ngrok"); }
        }
    }

    let public: Option<String> = if public_url.is_empty() { None } else { Some(public_url.clone()) };
    let session_info = SessionInfo {
        vnc_port, web_port,
        local_ip: local_ip.clone(),
        public_url: public.clone(),
        password: pass.into(),
        display: if is_wayland { "Wayland" } else { "X11" }.into(),
        has_ngrok: public.is_some(),
    };

    start_web_ui(&session_info);

    println!();
    println!("{}", "Desktop Share".style(style::Theme::HEADER));
    println!("{}", style::divider());
    println!("{}", style::label_value("VNC", &format!("{}:{}", local_ip, vnc_port)));
    println!("{}", style::label_value("Web UI", &format!("http://localhost:{}", web_port)));
    println!("{}", style::label_value("Password", pass));

    if let Some(ref url) = public {
        println!("{}", style::label_value("Public", url));
    } else {
        println!("{}", style::label_value("Public", "none (install ngrok)"));
    }
    println!("{}", style::divider());

    println!("\n{} Open {} in your browser.", "  ".dimmed(), format!("http://localhost:{}", web_port).style(style::Theme::ACCENT));
    if public.is_some() {
        println!("{} Share the Public URL — any VNC client can connect.", "  ".dimmed());
    }
    println!("{} Ctrl+C to stop.\n", "  ".dimmed());
    println!("{}", style::divider());

    let _ = child.wait();

    if let Some(pid) = ngrok_pid {
        let _ = std::process::Command::new("kill").arg(pid.to_string()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    }

    println!("\n{} Session stopped.", style::success(""));
}

struct SessionInfo {
    vnc_port: u16,
    web_port: u16,
    local_ip: String,
    public_url: Option<String>,
    password: String,
    display: String,
    has_ngrok: bool,
}

fn start_web_ui(info: &SessionInfo) {
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", info.web_port))
        .expect("Failed to bind web UI port");
    listener.set_nonblocking(true).ok();

    let local_ip = info.local_ip.clone();
    let public_url = info.public_url.clone();
    let password = info.password.clone();
    let display = info.display.clone();
    let vnc_port = info.vnc_port;
    let web_port = info.web_port;

    std::thread::spawn(move || {
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    use std::io::{Read, Write};
                    let mut buf = [0u8; 4096];
                    let _ = stream.read(&mut buf);

                    let public_section = if let Some(ref url) = public_url {
                        format!(r#"<div class="card public">
                            <div class="label">Public URL</div>
                            <div class="value highlight">{}</div>
                            <div class="note">Share this — any VNC client can connect</div>
                        </div>"#, url)
                    } else {
                        r#"<div class="card missing">
                            <div class="label">Public Access</div>
                            <div class="value">Not available</div>
                            <div class="note">Install ngrok: <code>sudo pacman -S ngrok</code></div>
                        </div>"#.to_string()
                    };

                    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="refresh" content="8">
<title>Proto Desktop Share</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{background:#0d1117;color:#c9d1d9;font-family:system-ui,sans-serif;min-height:100vh;display:flex;flex-direction:column;align-items:center;padding:40px 20px}}
h1{{color:#58a6ff;font-size:22px;margin-bottom:8px}}
.subtitle{{color:#8b949e;font-size:14px;margin-bottom:32px}}
.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px;width:100%;max-width:720px}}
.card{{background:#161b22;border:1px solid #30363d;border-radius:8px;padding:20px}}
.card.public{{border-color:#3fb950;background:#0d2b15}}
.card.missing{{border-color:#30363d;opacity:.6}}
.label{{font-size:12px;color:#8b949e;text-transform:uppercase;letter-spacing:.5px;margin-bottom:6px}}
.value{{font-size:20px;font-weight:600;word-break:break-all}}
.value.highlight{{color:#3fb950}}
.note{{font-size:12px;color:#484f58;margin-top:8px}}
.note code{{background:#21262d;padding:2px 6px;border-radius:3px;font-size:11px}}
.footer{{margin-top:32px;color:#484f58;font-size:12px;text-align:center}}
.status{{display:inline-block;width:8px;height:8px;background:#3fb950;border-radius:50%;margin-right:8px;box-shadow:0 0 8px #3fb95066;animation:pulse 2s infinite}}
@keyframes pulse{{0%,100%{{opacity:1}} 50%{{opacity:.5}}}}
</style></head>
<body>
<h1><span class="status"></span>Proto Desktop Share</h1>
<div class="subtitle">{} display — {} VNC server</div>
<div class="grid">
    <div class="card"><div class="label">VNC Address</div><div class="value">{}:{}</div><div class="note">Password: <code>{}</code></div></div>
    <div class="card"><div class="label">Web UI</div><div class="value">http://localhost:{}</div><div class="note">This page — auto-refreshes</div></div>
    {}
</div>
<div class="footer">Proto CLI · Viewer needs a VNC client (TigerVNC, RealVNC, Remmina)</div>
</body></html>"##,
                        display, display, local_ip, vnc_port, password,
                        web_port, public_section
                    );

                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        html.len(), html
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

fn fallback_local_vnc(ip: &str, port: u16) {
    println!("{}", "Local Network Access".style(style::Theme::HEADER));
    println!("{}", style::divider());
    println!("{}", style::label_value("Connect to", &format!("{}:{}", ip, port)));
    println!("{}", style::label_value("Password", "proto"));
    println!("{}", style::divider());
    println!("\n  {} Any VNC client can connect: {}", "▸".style(style::Theme::ACCENT), "TigerVNC, RealVNC, Remmina".dimmed());
    println!("  {} From terminal: {}", "▸".style(style::Theme::ACCENT), format!("vncviewer {}:{}", ip, port).dimmed());
}

fn get_local_ip() -> Option<String> {
    crate::utils::run_command_output("hostname", &["-I"])
        .ok()
        .and_then(|s| s.split_whitespace().next().map(|s| s.to_string()))
}

fn join(link: &str) {
    let sp = style::Spinner::new(&format!("Joining {}...", link));

    if link.starts_with("ssh ") {
        let addr = link.trim_start_matches("ssh ").trim();
        sp.done(&format!("Connecting to {}", addr));
        println!("\n{} Connecting via SSH...", "  ".dimmed());
        let _ = std::process::Command::new("ssh").arg(addr).status();
    } else if link.contains("tmate.io") || link.contains("sshx.io") || link.starts_with("http") {
        sp.done(&format!("Opening {}", link));
        println!("\n{} Open in browser: {}", "  ".dimmed(), link.style(style::Theme::ACCENT));
        if crate::utils::which("xdg-open") {
            let _ = std::process::Command::new("xdg-open").arg(link).status();
        }
    } else {
        sp.fail("Unrecognized link format.");
    }
}
