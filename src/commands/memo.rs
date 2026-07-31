use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum MemoAction {
    #[command(about = "Show all memos")]
    List,
    #[command(about = "Add a new memo")]
    Add {
        #[arg(required = true, value_name = "TEXT")]
        text: String,
    },
    #[command(about = "Clear all memos")]
    Clear,
}

pub fn run(action: &MemoAction) {
    match action {
        MemoAction::List => list(),
        MemoAction::Add { text } => add(text),
        MemoAction::Clear => clear(),
    }
}

fn memo_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_default().join(".proto")
}

fn list() {
    let path = memo_path();
    if !path.exists() {
        println!("{} No memos here yet.", "  ".dimmed());
        println!("  {}", "proto memo add \"your note here\"".style(style::Theme::MUTED));
        return;
    }

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.trim().is_empty() {
        println!("{} No memos.", "  ".dimmed());
        return;
    }

    println!("{} {}", "◆".style(style::Theme::ACCENT), format!("Memos — {}", path.to_string_lossy()).style(style::Theme::HEADER));
    println!("{}", style::divider());
    println!("{}", content);
    println!("{}", style::divider());
}

fn add(text: &str) {
    let path = memo_path();
    let now = chrono_now();

    let entry = format!("[{}] {}\n", now, text);

    let existing = if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };

    std::fs::write(&path, format!("{}{}", existing, entry)).unwrap();
    println!("{} {}", "✦".style(style::Theme::SUCCESS), text.style(style::Theme::ACCENT));
    println!("  {} ({})", "in".dimmed(), path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
}

fn clear() {
    use dialoguer::Confirm;
    let path = memo_path();
    if !path.exists() {
        println!("{} No memos to clear.", "  ".dimmed());
        return;
    }

    let confirm = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Delete all memos in this directory?")
        .default(false).interact().unwrap_or(false);

    if confirm {
        std::fs::remove_file(&path).unwrap();
        println!("{} Memos cleared.", style::success(""));
    } else {
        println!("{}", "Aborted.".style(style::Theme::MUTED));
    }
}

fn chrono_now() -> String {
    let dur = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let s = dur.as_secs() as i64;
    let days = s / 86400;
    let rem = s % 86400;
    let h = rem / 3600;
    let mi = (rem % 3600) / 60;
    format!("{:02}:{:02}", h, mi)
}
