use crate::style;
use owo_colors::OwoColorize;
use std::process::Command;

pub fn run(output: Option<String>, cmd: Vec<String>) {
    println!("{}", style::header("Asciicast"));
    println!("{}", style::divider());

    let asciinema = crate::utils::which("asciinema");
    let script = crate::utils::which("script");

    if asciinema {
        println!("  {} Using asciinema ...\n", style::muted(""));
        let mut args: Vec<&str> = vec!["rec"];
        if let Some(ref out) = output {
            args.push(out);
        }
        // If no command given, use shell
        if cmd.is_empty() {
            let status = Command::new("asciinema").args(&args).status();
            match status {
                Ok(s) if s.success() => {}
                _ => eprintln!("  {} asciinema exited with error.", style::error("")),
            }
        } else {
            let mut args: Vec<String> = vec!["rec".to_string()];
            if let Some(ref out) = output {
                args.push(out.clone());
            }
            args.push("--command".to_string());
            args.push(cmd.join(" "));
            let status = Command::new("asciinema").args(&args).status();
            match status {
                Ok(s) if s.success() => {}
                _ => eprintln!("  {} asciinema exited with error.", style::error("")),
            }
        }
    } else if script {
        let out = output.unwrap_or_else(|| "recording.cast".to_string());
        println!(
            "  {} Using script (no asciinema found). Output: {}\n",
            style::muted(""),
            out.style(style::Theme::VALUE)
        );
        if cmd.is_empty() {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            let _ = Command::new("script")
                .args(["-q", &out, "-c", &shell])
                .status();
        } else {
            let _ = Command::new("script")
                .args(["-q", &out, "-c", &cmd.join(" ")])
                .status();
        }
    } else {
        println!(
            "  {} Install {} or use:\n",
            style::warn("asciinema not found."),
            "asciinema".style(style::Theme::VALUE)
        );
        println!("    pacman -S asciinema");
        println!("    brew install asciinema\n");
        println!("  Then re-run: proto asciicast");
    }
}
