use clap::Subcommand;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum AiAction {
    #[command(about = "Start an interactive AI chat session")]
    Chat,
    #[command(about = "Configure AI provider, key, and personality")]
    Setup,
    #[command(about = "Summarize git log into a changelog")]
    Summarize {
        #[arg(value_name = "FROM", help = "Starting tag or commit")]
        from: Option<String>,
        #[arg(value_name = "TO", default_value = "HEAD", help = "Ending tag or commit")]
        to: String,
        #[arg(short, long, default_value = "CHANGELOG.md", value_name = "FILE")]
        output: String,
    },
    #[command(about = "Explain the last failed command using AI")]
    Explain,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub personality: String,
    pub custom_prompt: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            personality: "engineer".into(),
            custom_prompt: None,
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    crate::utils::config_dir().join("ai.toml")
}

pub fn load_config() -> AiConfig {
    let path = config_path();
    if path.exists() {
        std::fs::read_to_string(&path).ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AiConfig::default()
    }
}

pub fn personality_prompt(personality: &str, custom: Option<&str>) -> String {
    if personality == "custom" {
        return custom.unwrap_or("You are a helpful assistant.").to_string();
    }
    match personality {
        "furry" => "You are a cute protogen AI assistant. Be playful, use uwu occasionally, add :3 emotes, be friendly and warm. Keep answers helpful but adorable.".into(),
        "engineer" => "You are a precise software engineering assistant. Be technical, concise, and accurate. Use code examples. No fluff. Answer like a senior engineer code reviewing.".into(),
        "minimal" => "You are a terse assistant. Answer in the fewest words possible. No explanations unless asked.".into(),
        _ => "You are Proto, a helpful CLI assistant. Be friendly, concise, and practical. Use code blocks when showing commands.".into(),
    }
}

pub fn run(action: &AiAction) {
    match action {
        AiAction::Chat => chat(),
        AiAction::Setup => setup(),
        AiAction::Summarize { from, to, output } => summarize(from.as_deref(), to, output),
        AiAction::Explain => explain(),
    }
}

fn setup() {
    use dialoguer::{Confirm, Input, Select, Password};

    println!("{}", style::proto_banner());
    println!("{}\n", "AI Setup".style(style::Theme::HEADER));

    let providers = &["openai", "gemini"];
    let prov_idx = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("AI Provider")
        .items(providers)
        .default(0).interact().unwrap_or(0);

    let provider = providers[prov_idx].to_string();

    let default_model = match provider.as_str() {
        "openai" => "gpt-4o-mini",
        "gemini" => "gemini-2.0-flash",
        _ => "gpt-4o-mini",
    };

    let api_key: String = Password::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("API Key")
        .interact().unwrap();

    let model: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Model")
        .default(default_model.into())
        .interact_text().unwrap();

    let personalities = &[
        "engineer  — technical, precise, code-focused",
        "helpful   — friendly, concise, practical",
        "furry     — cute protogen, uwu, playful :3",
        "minimal   — terse, few words",
        "custom    — define your own prompt",
    ];
    let pers_keys = &["engineer", "helpful", "furry", "minimal", "custom"];

    let pers_idx = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Personality")
        .items(personalities)
        .default(0).interact().unwrap_or(0);

    let personality = pers_keys[pers_idx].to_string();

    let custom_prompt = if personality == "custom" {
        let cp: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Custom system prompt")
            .default("You are a helpful assistant.".into())
            .interact_text().unwrap();
        Some(cp)
    } else {
        None
    };

    let config = AiConfig { provider, api_key, model, personality, custom_prompt };
    let toml_str = toml::to_string_pretty(&config).unwrap();

    let dir = crate::utils::config_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(config_path(), toml_str).unwrap();

    println!("\n{} AI configuration saved!", style::success(""));
    println!("  Try: {}", "proto ai".style(style::Theme::ACCENT));
}

fn chat() {
    let config = load_config();
    if config.api_key.is_empty() {
        eprintln!("{} AI not configured. Run: {}", style::error(""), "proto ai setup".style(style::Theme::ACCENT));
        return;
    }

    let system = personality_prompt(&config.personality, config.custom_prompt.as_deref());
    println!("{} {}\n", "◆".style(style::Theme::ACCENT), format!("Proto AI ({})", config.model).style(style::Theme::HEADER));
    println!("{} Type /quit to exit, /clear to reset context.", "  ".dimmed());
    println!("{}", style::divider());

    let mut messages: Vec<(String, String)> = vec![("system".into(), system)];

    loop {
        use std::io::{Write, BufRead};
        print!("{} ", "you ›".style(style::Theme::ACCENT).bold());
        let _ = std::io::stdout().flush();

        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        let input = line.trim().to_string();

        if input.is_empty() { continue; }
        if input == "/quit" || input == "/exit" { break; }
        if input == "/clear" {
            messages.truncate(1);
            println!("{} Context cleared.", "  ".dimmed());
            continue;
        }

        messages.push(("user".into(), input.clone()));

        let sp = style::Spinner::new("Thinking...");
        match call_ai(&config, &messages) {
            Ok(response) => {
                sp.done("");
                println!("\n{} {}\n", "bot ›".style(style::Theme::SUCCESS).bold(), response.trim());
                messages.push(("assistant".into(), response));
            }
            Err(e) => {
                sp.fail(&e);
            }
        }
    }
    println!("\n{} Goodbye! :3", "  ".dimmed());
}

fn summarize(from: Option<&str>, to: &str, output: &str) {
    let config = load_config();
    if config.api_key.is_empty() {
        eprintln!("{} AI not configured. Run: {}", style::error(""), "proto ai setup".style(style::Theme::ACCENT));
        return;
    }

    let sp = style::Spinner::new("Collecting git log...");

    let range = if let Some(f) = from { format!("{}..{}", f, to) } else { to.to_string() };
    let log = crate::utils::run_command_output("git", &["log", "--oneline", &range]).unwrap_or_default();

    if log.is_empty() {
        sp.fail("No commits found in range.");
        return;
    }

    let commit_count = log.lines().count();
    sp.update(&format!("Sending {} commits to AI...", commit_count));

    let system = "You are a changelog generator. Output a clean, well-formatted CHANGELOG.md in Keep a Changelog format. Group commits into Added, Changed, Fixed, Removed. Be concise. Only include the changelog, no preamble.".to_string();

    let prompt = format!(
        "Generate a changelog from these {} git commits:\n\n{}",
        commit_count,
        log
    );

    let messages = vec![
        ("system".into(), system),
        ("user".into(), prompt),
    ];

    match call_ai(&config, &messages) {
        Ok(response) => {
            std::fs::write(output, &response).unwrap();
            sp.done(&format!("Changelog written to {}", output));
            println!("\n{}", response.lines().take(20).collect::<Vec<_>>().join("\n"));
            if response.lines().count() > 20 { println!("..."); }
        }
        Err(e) => {
            sp.fail(&e);
        }
    }
}

fn explain() {
    let config = load_config();
    if config.api_key.is_empty() {
        eprintln!("{} AI not configured. Run: {}", style::error(""), "proto ai setup".style(style::Theme::ACCENT));
        return;
    }

    let hist_file = dirs::home_dir().unwrap_or_default().join(".proto_last_error");

    if !hist_file.exists() {
        eprintln!("{} No recent error captured.", style::warn(""));
        eprintln!("  Run a command that fails, then run {}", "proto ai explain".style(style::Theme::ACCENT));
        return;
    }

    let error_text = std::fs::read_to_string(&hist_file).unwrap_or_default();
    if error_text.trim().is_empty() {
        eprintln!("{} No error content found.", style::warn(""));
        return;
    }

    println!("{} Analyzing last error...", "◆".style(style::Theme::ACCENT));

    let system = "You are a terminal error explainer. Given stderr output from a failed command, explain what went wrong in plain English and suggest the most likely fix. Be concise. Use code blocks for commands.".to_string();

    let prompt = format!("This command failed with the following error output:\n\n```\n{}\n```\n\nExplain what happened and how to fix it.", error_text);

    let messages = vec![
        ("system".into(), system),
        ("user".into(), prompt),
    ];

    let sp = style::Spinner::new("Asking AI...");
    match call_ai(&config, &messages) {
        Ok(response) => {
            sp.done("");
            println!("\n{}", response);
        }
        Err(e) => {
            sp.fail(&e);
        }
    }
}

fn call_ai(config: &AiConfig, messages: &[(String, String)]) -> Result<String, String> {
    match config.provider.as_str() {
        "openai" => call_openai(config, messages),
        "gemini" => call_gemini(config, messages),
        _ => Err(format!("Unknown provider: {}", config.provider)),
    }
}

fn call_openai(config: &AiConfig, messages: &[(String, String)]) -> Result<String, String> {
    let msgs: Vec<serde_json::Value> = messages.iter().map(|(role, content)| {
        serde_json::json!({"role": role, "content": content})
    }).collect();

    let body = serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "temperature": 0.7,
        "max_tokens": 2048,
    });

    let resp = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("API error: {}", e))?;

    let json: serde_json::Value = resp.into_json().map_err(|e| format!("Parse error: {}", e))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected response: {}", json))
}

fn call_gemini(config: &AiConfig, messages: &[(String, String)]) -> Result<String, String> {
    let contents: Vec<serde_json::Value> = messages.iter().filter(|(r, _)| r != "system").map(|(role, content)| {
        let r = if role == "assistant" { "model" } else { "user" };
        serde_json::json!({
            "role": r,
            "parts": [{"text": content}]
        })
    }).collect();

    let system_instruction = messages.iter().find(|(r, _)| r == "system").map(|(_, c)| c.clone());

    let mut body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "temperature": 0.7,
            "maxOutputTokens": 2048,
        }
    });

    if let Some(sys) = &system_instruction {
        body["systemInstruction"] = serde_json::json!({
            "parts": [{"text": sys}]
        });
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        config.model, config.api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("API error: {}", e))?;

    let json: serde_json::Value = resp.into_json().map_err(|e| format!("Parse error: {}", e))?;

    json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected response: {}", json))
}
