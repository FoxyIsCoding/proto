use crate::style;
use owo_colors::OwoColorize;

pub fn run(query: String, source: Option<String>) {
    println!("{}", style::header("Search Docs"));
    println!("{}", style::divider());

    let source = source.unwrap_or_else(|| "cheat.sh".to_string());
    let query_enc = query.trim().replace(' ', "+");
    let url = match source.as_str() {
        "tldr" => format!("https://tldr.sh/{}?format=raw", query_enc),
        "cheat" | "cheat.sh" | _ => format!("https://cheat.sh/{}?T", query_enc),
    };
    println!(
        "  {} {} {}\n",
        style::muted("Searching"),
        source.style(style::Theme::VALUE),
        format!("\"{}\"", query).style(style::Theme::VALUE)
    );

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(10))
        .build();

    match agent.get(&url).call() {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_else(|e| format!("Error: {}", e));
            for line in body.lines() {
                let t = line.trim();
                if t.starts_with('#') {
                    println!("  {}", t.style(style::Theme::HEADER));
                } else if t.starts_with('>') || t.starts_with("//") {
                    println!("  {}", t.style(style::Theme::MUTED));
                } else if t.starts_with("$ ") || t.starts_with("  ") {
                    println!("  {}", t.style(style::Theme::VALUE));
                } else if !t.is_empty() {
                    println!("  {}", t);
                } else {
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!(
                "  {} Failed to fetch: {}",
                style::error(""),
                e
            );
        }
    }
    println!();
}
