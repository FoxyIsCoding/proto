use crate::style;
use owo_colors::OwoColorize;
use std::net::{SocketAddr, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn run(spec: &str, retries: usize, interval: u64) {
    let (local, ssh_target, remote) = match parse_spec(spec) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "{} {} Use: {}",
                style::error(""),
                e,
                "proto port-forward 8080:user@host:5432".style(style::Theme::ACCENT)
            );
            return;
        }
    };

    if !crate::utils::which("ssh") {
        eprintln!("{} ssh not found on PATH.", style::error(""));
        return;
    }

    println!("{}", style::header("SSH Port Forward"));
    println!("{}", style::divider());
    println!(
        "  {}",
        style::label_value(
            "Forward",
            &format!("127.0.0.1:{} → {}:{}", local, ssh_target, remote)
        )
    );
    if retries == 0 {
        println!(
            "  {}",
            style::label_value("Retries", "unlimited (press Ctrl+C to stop)")
        );
    } else {
        println!("  {}", style::label_value("Retries", &retries.to_string()));
    }

    let local = local as u16;
    if port_open(local) {
        println!(
            "  {} Local port {} is already in use.",
            style::warn(""),
            local
        );
    }

    let stop = Arc::new(AtomicBool::new(false));
    let mon = Arc::clone(&stop);
    let health = std::thread::spawn(move || {
        let mut last = false;
        loop {
            let ok = port_open(local);
            if ok != last {
                if ok {
                    println!(
                        "  {} Forward is UP ({}:{} reachable)",
                        "▲".style(style::Theme::SUCCESS),
                        "127.0.0.1",
                        local
                    );
                } else {
                    println!(
                        "  {} Forward is DOWN ({}:{})",
                        "▼".style(style::Theme::ERROR),
                        "127.0.0.1",
                        local
                    );
                }
                last = ok;
            }
            if mon.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_secs(interval.max(1)));
        }
    });

    let mut attempt: usize = 0;
    let mut quick_exits = 0usize;
    loop {
        if attempt > 0 {
            if retries != 0 && attempt > retries {
                println!("  {} Max retries reached, giving up.", style::error(""));
                break;
            }
            println!(
                "  {} Connection dropped, reconnecting in 3s... (attempt {}{})",
                "↻".style(style::Theme::WARN),
                attempt,
                if retries == 0 {
                    String::new()
                } else {
                    format!("/{}", retries)
                }
            );
            std::thread::sleep(Duration::from_secs(3));
        }
        attempt += 1;

        let mut child = match spawn_ssh(local, ssh_target, remote) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  {} Failed to launch ssh: {}", style::error(""), e);
                break;
            }
        };

        let started = Instant::now();
        let mut alive = true;
        while alive {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let quick = started.elapsed() < Duration::from_secs(10);
                    if quick {
                        quick_exits += 1;
                    } else {
                        quick_exits = 0;
                    }
                    if quick_exits >= 3 {
                        println!(
                            "  {} ssh keeps exiting immediately — check your auth key or host.",
                            style::warn("")
                        );
                    }
                    if status.code() == Some(255) {
                        println!("  {} ssh error (code 255).", style::error(""));
                    }
                    alive = false;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(500)),
                Err(_) => {
                    alive = false;
                }
            }
        }
        let _ = child.kill();
    }

    stop.store(true, Ordering::Relaxed);
    let _ = health.join();
}

fn parse_spec(spec: &str) -> Result<(u16, &str, u16), String> {
    let (local, rest) = spec
        .split_once(':')
        .ok_or_else(|| format!("Invalid spec '{}'", spec))?;
    let (host, remote) = rest
        .rsplit_once(':')
        .ok_or_else(|| format!("Invalid spec '{}'", spec))?;
    if host.is_empty() || remote.is_empty() {
        return Err(format!("Invalid spec '{}'", spec));
    }
    let l: u16 = local
        .parse()
        .map_err(|_| format!("Invalid local port '{}'", local))?;
    let r: u16 = remote
        .parse()
        .map_err(|_| format!("Invalid remote port '{}'", remote))?;
    Ok((l, host, r))
}

fn spawn_ssh(local: u16, ssh_target: &str, remote: u16) -> std::io::Result<Child> {
    let listener = format!("{}:127.0.0.1:{}", local, remote);
    Command::new("ssh")
        .args([
            "-N",
            "-T",
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=3",
            "-o",
            "ExitOnForwardFailure=yes",
            "-o",
            "ConnectTimeout=10",
            "-L",
            &listener,
            ssh_target,
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

fn port_open(port: u16) -> bool {
    let addr: SocketAddr = match format!("127.0.0.1:{}", port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(800)).is_ok()
}
