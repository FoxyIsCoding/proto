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
    #[command(about = "Minecraft utilities (resource packs & servers)")]
    Mc {
        #[command(subcommand)]
        action: commands::mc::McAction,
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
        Some(Commands::System) => commands::system::run(),
        Some(Commands::Pkg { action }) => commands::pkg::run(&action),
        Some(Commands::Git { action }) => commands::git::run(&action),
        Some(Commands::Setup) => commands::setup::run(),
        Some(Commands::Mc { action }) => commands::mc::run(&action),
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
