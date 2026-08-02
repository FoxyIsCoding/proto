use crate::style;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    text: String,
    done: bool,
}

fn todo_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("proto")
        .join("todos.json")
}

fn load() -> Vec<Task> {
    let path = todo_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save(tasks: &[Task]) {
    let path = todo_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(tasks).unwrap_or_default();
    let _ = std::fs::write(&path, json);
}

pub fn run(action: &str, text: Vec<String>, id: usize) {
    println!("{}", style::header("Todo"));
    println!("{}", style::divider());

    match action {
        "add" => {
            let text = text.join(" ");
            if text.is_empty() {
                println!("  {} Usage: proto todo add <task text>", style::muted(""));
                return;
            }
            let mut tasks = load();
            let new_id = tasks.last().map(|t| t.id + 1).unwrap_or(1);
            tasks.push(Task {
                id: new_id,
                text,
                done: false,
            });
            save(&tasks);
            println!(
                "  {} Added task #{}.",
                style::success(""),
                new_id.style(style::Theme::VALUE)
            );
        }
        "done" => {
            if id == 0 {
                println!("  {} Usage: proto todo done <ID>", style::muted(""));
                return;
            }
            let mut tasks = load();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
                t.done = true;
                save(&tasks);
                println!(
                    "  {} Marked #{} as done.",
                    style::success(""),
                    id.style(style::Theme::VALUE)
                );
            } else {
                println!("  {} Task #{} not found.", style::warn(""), id);
            }
        }
        "remove" => {
            if id == 0 {
                println!("  {} Usage: proto todo remove <ID>", style::muted(""));
                return;
            }
            let mut tasks = load();
            if let Some(pos) = tasks.iter().position(|t| t.id == id) {
                tasks.remove(pos);
                save(&tasks);
                println!(
                    "  {} Removed task #{}.",
                    style::success(""),
                    id.style(style::Theme::VALUE)
                );
            } else {
                println!("  {} Task #{} not found.", style::warn(""), id);
            }
        }
        "list" | _ => {
            let tasks = load();
            if tasks.is_empty() {
                println!("  {} No tasks yet. Use {} to add one.", style::muted(""), "proto todo add <text>".style(style::Theme::VALUE));
                return;
            }
            let pending: Vec<&Task> = tasks.iter().filter(|t| !t.done).collect();
            let completed: Vec<&Task> = tasks.iter().filter(|t| t.done).collect();

            if !pending.is_empty() {
                println!("  {} Pending:", style::label_value("", ""));
                for t in &pending {
                    println!(
                        "    #{}  {}",
                        t.id.to_string().style(style::Theme::VALUE),
                        t.text
                    );
                }
                println!();
            }
            if !completed.is_empty() {
                println!("  {} Done:", style::muted(""));
                for t in &completed {
                    println!(
                        "    #{}  {} {}",
                        t.id.to_string().style(style::Theme::MUTED),
                        t.text.style(style::Theme::MUTED),
                        "✔".green()
                    );
                }
                println!();
            }
            println!(
                "  {} {} tasks ({} pending, {} done)",
                style::muted(""),
                tasks.len().style(style::Theme::VALUE),
                pending.len().style(style::Theme::VALUE),
                completed.len().style(style::Theme::MUTED)
            );
        }
    }
    println!();
}
