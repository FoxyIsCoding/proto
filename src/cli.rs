use clap::{Parser, Subcommand};
use crate::commands;

#[derive(Parser, Debug)]
#[command(
    name = "proto",
    version,
    about = "Your friendly protogen CLI companion",
    long_about = None,
    disable_help_subcommand = true,
    disable_help_flag = true,
    disable_version_flag = true,
    arg_required_else_help = false,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue, global = false)]
    pub help_flag: bool,

    #[arg(short = 'V', long = "version", action = clap::ArgAction::SetTrue, global = false)]
    pub version_flag: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(about = "Show help for all commands or a specific command")]
    Help {
        #[arg(value_name = "COMMAND")]
        command: Option<String>,
    },
    #[command(about = "Display beautiful system information")]
    System,
    #[command(about = "Cross-distro package manager wrapper")]
    Pkg {
        #[command(subcommand)]
        action: commands::pkg::PkgAction,
    },
    #[command(about = "Git workflow enhancements")]
    Git {
        #[command(subcommand)]
        action: commands::git::GitAction,
    },
    #[command(about = "Interactive first-time configuration wizard")]
    Setup,
    #[command(about = "Interactive shell alias builder (bash/zsh/fish)")]
    Alias {
        #[command(subcommand)]
        action: commands::alias::AliasAction,
    },
    #[command(about = "Minecraft utilities (resource packs & servers)")]
    Mc {
        #[command(subcommand)]
        action: commands::mc::McAction,
    },
    #[command(name = "share-session", about = "Share terminal session via tmate/tmux")]
    ShareSession {
        #[command(subcommand)]
        action: commands::share::ShareAction,
    },
    #[command(about = "Network status monitoring tools")]
    Status {
        #[command(subcommand)]
        action: commands::status::StatusAction,
    },
    #[command(about = "Discord bot & quest utilities")]
    Discord {
        #[command(subcommand)]
        action: commands::discord::DiscordAction,
    },
    #[command(about = "Convert between units (time, length, weight, digital, etc.)")]
    Convert {
        #[arg(required = true, value_name = "VALUE", help = "Value with unit (e.g. 6m, 10.5km, 500ms, 2GB)")]
        input: String,
        #[arg(value_name = "TO", help = "Target unit (e.g. cm, min, MB, F)")]
        to: Option<String>,
    },
    #[command(about = "Encode, decode, hash, and generate cryptographic values")]
    Encrypt {
        #[command(subcommand)]
        action: commands::encrypt::EncryptAction,
    },
    #[command(about = "Project diagnostics, port management, cleanup, and snapshots")]
    App {
        #[command(subcommand)]
        action: commands::app::AppAction,
    },
    #[command(about = "AI chat, changelog generation, and error explainer")]
    Ai {
        #[command(subcommand)]
        action: commands::ai::AiAction,
    },
    #[command(name = "copy-ctx", about = "Bundle git repo source files into an LLM-ready clipboard context")]
    CopyCtx,
    #[command(about = "Location-aware scratchpad memos")]
    Memo {
        #[command(subcommand)]
        action: commands::memo::MemoAction,
    },
}

pub fn run(cli: Cli) {
    if cli.version_flag {
        print_version();
        return;
    }

    if cli.help_flag {
        print_short_help();
        return;
    }

    match cli.command {
        Some(Commands::Help { command }) => {
            match command {
                Some(cmd) => commands::help::run(&commands::help::HelpAction::For { command: cmd }),
                None => commands::help::run(&commands::help::HelpAction::All),
            }
        }
        Some(Commands::ShareSession { action }) => commands::share::run(&action),
        Some(Commands::Alias { action }) => commands::alias::run(&action),
        Some(Commands::System) => commands::system::run(),
        Some(Commands::Pkg { action }) => commands::pkg::run(&action),
        Some(Commands::Git { action }) => commands::git::run(&action),
        Some(Commands::Setup) => commands::setup::run(),
        Some(Commands::Mc { action }) => commands::mc::run(&action),
        Some(Commands::Status { action }) => commands::status::run(&action),
        Some(Commands::Discord { action }) => commands::discord::run(&action),
        Some(Commands::Convert { input, to }) => commands::convert::run(&commands::convert::ConvertAction::Run { input, to }),
        Some(Commands::Encrypt { action }) => commands::encrypt::run(&action),
        Some(Commands::App { action }) => commands::app::run(&action),
        Some(Commands::Ai { action }) => commands::ai::run(&action),
        Some(Commands::CopyCtx) => commands::copyctx::run(),
        Some(Commands::Memo { action }) => commands::memo::run(&action),
        None => {
            commands::help::run(&commands::help::HelpAction::All);
        }
    }
}

fn print_version() {
    use owo_colors::OwoColorize;
    use crate::style;

    println!("{}", style::proto_banner());
    println!(
        "{} {}",
        "proto".style(style::Theme::HEADER).bold(),
        env!("CARGO_PKG_VERSION").style(style::Theme::MUTED)
    );
    println!("{}", "Your friendly protogen CLI companion".style(style::Theme::MUTED));
}

fn print_short_help() {
    commands::help::run(&commands::help::HelpAction::All);
}
