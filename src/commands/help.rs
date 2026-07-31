use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum HelpAction {
    #[command(about = "Show help for all commands")]
    All,
    #[command(about = "Show help for a specific command")]
    For {
        #[arg(value_name = "COMMAND")]
        command: String,
    },
}

pub fn run(action: &HelpAction) {
    match action {
        HelpAction::All => print_general_help(),
        HelpAction::For { command } => print_command_help(command),
    }
}

fn print_general_help() {
    println!("{}", style::proto_banner());
    println!("{}\n", "Proto CLI".style(style::Theme::HEADER).bold());
    println!("{}  {}", "◆".style(style::Theme::ACCENT), "Your friendly protogen CLI companion\n".style(style::Theme::MUTED));

    println!("{}", "USAGE:".style(style::Theme::HEADER));
    println!("  {} <command> [options]\n", "proto".style(style::Theme::ACCENT));

    println!("{}", "COMMANDS:".style(style::Theme::HEADER));
    print_cmd("help", "[command]", "Show this help or help for a specific command");
    print_cmd("system", "", "Display beautiful system information");
    print_cmd("alias", "create|list|remove", "Interactive shell alias builder");
    print_cmd("share-session", "create|join", "Share terminal via tmate/tmux");
    print_cmd("pkg", "<action>", "Cross-distro package manager wrapper");
    print_cmd("git", "<action>", "Git workflow enhancements");
    print_cmd("setup", "", "Interactive configuration wizard");
    print_cmd("mc", "resource_pack|server", "Minecraft resource packs & servers");
    print_cmd("status", "ping|monitor|serve|report", "Network monitoring tools");
    print_cmd("discord", "bot|quest", "Discord bot creator & tools");
    print_cmd("app", "doctor|port|nuke|snap", "Project diagnostics & cleanup");
    print_cmd("ai", "chat|setup|summarize|explain", "AI assistant & changelog generator");
    print_cmd("copy-ctx", "", "Bundle repo into LLM-ready clipboard context");
    print_cmd("memo", "add|list|clear", "Location-aware scratchpad notes");

    println!("\n{}", "FLAGS:".style(style::Theme::HEADER));
    print_cmd("--version", "", "Print version and exit");
    print_cmd("--help", "", "Print help information");

    println!("\n{} {}\n", "Run".style(style::Theme::MUTED), "'proto help <command>' for more info.".style(style::Theme::MUTED));
}

fn print_command_help(command: &str) {
    match command.to_lowercase().as_str() {
        "help" => {
            println!("{}", "proto help".style(style::Theme::HEADER));
            println!("  Display help for all commands or a specific command.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto help              Show general help");
            println!("  proto help <command>    Show help for a command\n");
            println!("{}", "EXAMPLES:".style(style::Theme::HEADER));
            println!("  proto help system       Show system command help");
            println!("  proto help pkg          Show package manager help");
        }
        "system" => {
            println!("{}", "proto system".style(style::Theme::HEADER));
            println!("  Display a beautiful overview of your system.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto system\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  OS, kernel, architecture, CPU, RAM, disk usage,");
            println!("  uptime, DE/WM, shell, terminal, and package count.");
        }
        "alias" => {
            println!("{}", "proto alias".style(style::Theme::HEADER));
            println!("  Interactive shell alias builder for bash, zsh, and fish.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto alias create    Build a new alias interactively");
            println!("  proto alias list      Show all Proto-managed aliases");
            println!("  proto alias remove <NAME>  Remove an alias\n");
            println!("{}", "CREATE FLOW:".style(style::Theme::HEADER));
            println!("  1. Enter alias name + command + description");
            println!("  2. Choose target shells (multi-select)");
            println!("  3. Choose permanent (writes to .bashrc/.zshrc/config.fish)");
            println!("     or session-only");
        }
        "share-session" => {
            println!("{}", "proto share-session".style(style::Theme::HEADER));
            println!("  Share your terminal or desktop for pair programming.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto share-session create [--backend sshx|tmate|tmux|vnc]");
            println!("  proto share-session join <LINK>\n");
            println!("{}", "BACKENDS:".style(style::Theme::HEADER));
            println!("  sshx  — web link, viewer opens in browser (term only)");
            println!("  tmate — SSH + web link, works remotely (term only)");
            println!("  vnc   — full desktop! x11vnc/wayvnc + ngrok tunnel");
            println!("  tmux  — local-only, teammate must SSH in\n");
            println!("{}", "INSTALL:".style(style::Theme::HEADER));
            println!("  cargo install sshx");
            println!("  sudo pacman -S tmate tmux x11vnc wayvnc ngrok");
        }
        "pkg" => {
            println!("{}", "proto pkg".style(style::Theme::HEADER));
            println!("  Unified cross-distro package manager wrapper.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto pkg install <pkg>     Install a package");
            println!("  proto pkg search <query>    Search repositories");
            println!("  proto pkg remove <pkg>      Remove a package");
            println!("  proto pkg update [pkg]      Update all or specific packages");
            println!("  proto pkg list              List installed packages\n");
            println!("{}", "BUILD:".style(style::Theme::HEADER));
            println!("  proto pkg build pack create Interactive pack config creator");
            println!("  proto pkg build pack edit   Edit existing pack config");
            println!("  proto pkg build pack build  Generate portable installer");
            println!("  proto pkg build pack test   Dry-run pack config\n");
            println!("{}", "SUPPORTED:".style(style::Theme::HEADER));
            println!("  pacman, yay, paru, apt, dnf, zypper, apk");
        }
        "git" => {
            println!("{}", "proto git".style(style::Theme::HEADER));
            println!("  Git workflow enhancements.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto git log               Pretty git log with graph");
            println!("  proto git stats             Repository statistics");
            println!("  proto git save <msg>        Quick WIP commit (add all + commit)");
            println!("  proto git undo              Undo last commit (soft reset)");
            println!("  proto git branch            Show branches with info");
        }
        "setup" => {
            println!("{}", "proto setup".style(style::Theme::HEADER));
            println!("  Interactive first-time configuration wizard.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto setup\n");
            println!("{}", "CONFIGURES:".style(style::Theme::HEADER));
            println!("  Default package manager, color preferences,");
            println!("  shell completions, and more.");
        }
        "mc" => {
            println!("{}", "proto mc".style(style::Theme::HEADER));
            println!("  Minecraft utilities.\n");
            println!("{}", "RESOURCE PACK:".style(style::Theme::HEADER));
            println!("  proto mc resource_pack create    Create a new resource pack");
            println!("  proto mc resource_pack fetch     List Minecraft versions + stats");
            println!("  proto mc resource_pack pack      Pack folder into a .zip\n");
            println!("{}", "SERVER:".style(style::Theme::HEADER));
            println!("  proto mc server create           Interactive server setup wizard");
            println!("  proto mc server ping <ip[:port]> Check if a server is online");
            println!("  proto mc server status <ip[:port]> Detailed server info + players\n");
            println!("{}", "CREATE OPTIONS:".style(style::Theme::HEADER));
            println!("  --version VERSION   MC version (default: 1.21.1)");
            println!("  --name NAME         Pack name (default: Resource Pack)");
            println!("  --clean BOOL        Bare bones only (default: true)");
        }
        "status" => {
            println!("{}", "proto status".style(style::Theme::HEADER));
            println!("  Network monitoring tools.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto status ping <ip[:port]>     Check if a host is reachable");
            println!("  proto status monitor <ip[:port]>  Live monitoring with uptime stats");
            println!("  proto status serve <ips...>       Host a dark web dashboard");
            println!("  proto status report <ip[:port]>   Generate a human-readable report\n");
            println!("{}", "OPTIONS:".style(style::Theme::HEADER));
            println!("  -n, --interval SECONDS    Poll interval (default: 5)");
            println!("  -p, --port PORT           Dashboard port (default: 5050)");
            println!("  -o, --output FILE         Report output path");
        }
        "discord" => {
            println!("{}", "proto discord".style(style::Theme::HEADER));
            println!("  Discord bot & quest utilities.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto discord bot create   Interactive bot project generator");
            println!("  proto discord quest        Quest completion injector (WIP)\n");
            println!("{}", "BOT CREATE OPTIONS:".style(style::Theme::HEADER));
            println!("  --language    python|rust|javascript|typescript|csharp|cpp");
            println!("  --template    slash_command|prefix|repeater|counter|none");
            println!("\n{} Templates include full runnable code, env config, and deps.", "  ".dimmed());
        }
        "app" => {
            println!("{}", "proto app".style(style::Theme::HEADER));
            println!("  Project diagnostics and cleanup tools.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto app doctor              Audit deps, .env, and ports");
            println!("  proto app port release <PORT>  Find and kill port usage");
            println!("  proto app nuke [--skip]        Purge build artifacts");
            println!("  proto app snap <NAME> create   Snapshot git state");
            println!("  proto app snap <NAME> view     Inspect a snapshot");
            println!("  proto app snap <NAME> restore  Restore a snapshot");
            println!("  proto app snap <NAME> delete   Delete a snapshot");
            println!("  proto app snap                 List all snapshots");
        }
        "ai" => {
            println!("{}", "proto ai".style(style::Theme::HEADER));
            println!("  AI assistant powered by OpenAI or Gemini.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto ai                  Interactive chat session");
            println!("  proto ai setup            Configure API key + personality");
            println!("  proto ai summarize [FROM] [TO]  Generate CHANGELOG.md from git log");
            println!("  proto ai explain          Explain the last failed command\n");
            println!("{}", "PERSONALITIES:".style(style::Theme::HEADER));
            println!("  engineer, helpful, furry, minimal, custom");
        }
        "copy-ctx" => {
            println!("{}", "proto copy-ctx".style(style::Theme::HEADER));
            println!("  Scans git repo, opens a file picker, bundles selected files\n");
            println!("  into Markdown code blocks and copies to clipboard.\n");
            println!("  Paste directly into ChatGPT, Claude, or any LLM.");
        }
        "memo" => {
            println!("{}", "proto memo".style(style::Theme::HEADER));
            println!("  Location-aware scratchpad stored in .proto file.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto memo list         Show all memos");
            println!("  proto memo add <text>   Append a memo with timestamp");
            println!("  proto memo clear        Delete all memos (with confirmation)");
        }
        other => {
            println!("{} Unknown command: '{}'", style::error(""), other.style(style::Theme::ACCENT));
            println!("Run {} to see all available commands.", "proto help".style(style::Theme::ACCENT));
        }
    }
}

fn print_cmd(name: &str, args: &str, desc: &str) {
    let name_part = if args.is_empty() {
        format!("  {}          ", name.style(style::Theme::ACCENT))
    } else {
        format!("  {} {}", name.style(style::Theme::ACCENT), args.style(style::Theme::BOLD))
    };
    println!("{}{}", format!("{:40}", name_part), desc.style(style::Theme::MUTED));
}
