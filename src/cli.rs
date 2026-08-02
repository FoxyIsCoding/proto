use crate::commands;
use clap::{Parser, Subcommand};

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
    #[command(
        name = "share-session",
        about = "Share terminal session via tmate/tmux"
    )]
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
        #[arg(
            required = true,
            value_name = "VALUE",
            help = "Value with unit (e.g. 6m, 10.5km, 500ms, 2GB)"
        )]
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
    #[command(
        name = "copy-ctx",
        about = "Bundle git repo source files into an LLM-ready clipboard context"
    )]
    CopyCtx,
    #[command(about = "Location-aware scratchpad memos")]
    Memo {
        #[command(subcommand)]
        action: commands::memo::MemoAction,
    },
    #[command(about = "Scan shell history and logs for leaked secrets")]
    Secret {
        #[command(subcommand)]
        action: commands::secret::SecretAction,
    },
    #[command(about = "Print incoming webhooks as formatted JSON")]
    Webhook {
        #[command(subcommand)]
        action: commands::webhook::WebhookAction,
    },
    #[command(
        name = "pr-prep",
        about = "Run tests, lint, format, and open the PR page"
    )]
    PrPrep {
        #[arg(long, help = "Skip running the test suite")]
        skip_tests: bool,
        #[arg(long, help = "Skip running the linter")]
        skip_lint: bool,
        #[arg(long, help = "Skip auto-formatting")]
        skip_fmt: bool,
        #[arg(long, help = "Don't open the PR page in a browser")]
        no_open: bool,
    },
    #[command(
        name = "pr-checkout",
        about = "Check out a GitHub/GitLab pull request locally"
    )]
    PrCheckout {
        #[arg(required = true, value_name = "PR#|URL")]
        target: String,
    },
    #[command(
        name = "git-who-broke",
        about = "Bisect to find the commit that broke the tests"
    )]
    GitWhoBroke {
        #[arg(
            value_name = "TEST_COMMAND",
            num_args = 0..,
            allow_hyphen_values = true,
            trailing_var_arg = true,
            help = "Test command (default: auto-detected, e.g. cargo test)"
        )]
        command: Vec<String>,
    },
    #[command(
        name = "git-impact",
        about = "Calculate branch risk score before merging"
    )]
    GitImpact,
    #[command(
        name = "git-catchup",
        about = "Show what changed upstream since your last pull"
    )]
    GitCatchup,
    #[command(
        name = "share",
        about = "Upload a file to a temporary, self-expiring host"
    )]
    Share {
        #[arg(required = true, value_name = "FILE")]
        file: String,
    },
    #[command(about = "Find exact duplicate files by content hash")]
    Dedupe {
        #[arg(
            value_name = "DIR",
            default_value = ".",
            help = "Directory to scan (default: current directory)"
        )]
        dir: String,
    },
    #[command(about = "Compress images and videos in place")]
    Media {
        #[command(subcommand)]
        action: commands::media::MediaAction,
    },
    #[command(about = "Download videos and music via yt-dlp")]
    Download {
        #[command(subcommand)]
        action: commands::download::DownloadAction,
    },
    #[command(about = "Battery health diagnostics for laptops")]
    Battery {
        #[arg(long, help = "Stream to the panel webserver")]
        serve: bool,
        #[arg(long, default_value_t = 5, help = "Interval in seconds")]
        interval: u64,
        #[arg(long, default_value_t = crate::panel::default_port(), help = "Panel port")]
        port: u16,
    },
    #[command(
        name = "kill-heavy",
        about = "Find and interactively kill high CPU/RAM processes"
    )]
    KillHeavy {
        #[arg(long, default_value_t = 10.0, help = "Minimum CPU %% to flag")]
        cpu: f64,
        #[arg(long, default_value_t = 512, help = "Minimum RSS in MB to flag")]
        mem: u64,
        #[arg(long, help = "Show every process, not just heavy ones")]
        all: bool,
        #[arg(long, help = "Send the scan to the panel webserver")]
        serve: bool,
        #[arg(long, default_value_t = crate::panel::default_port(), help = "Panel port")]
        port: u16,
    },
    #[command(about = "Manage the Proto CLI itself (update, uninstall, reset)")]
    Manage {
        #[command(subcommand)]
        action: commands::manage::ManageAction,
    },
    #[command(about = "Interactive dashboard of listening ports")]
    Ports {
        #[arg(long, help = "Stream to the panel webserver")]
        serve: bool,
        #[arg(long, default_value_t = crate::panel::default_port(), help = "Panel port")]
        port: u16,
    },
    #[command(
        name = "tree-view",
        about = "ASCII folder tree that respects .gitignore"
    )]
    TreeView {
        #[arg(value_name = "DIR", default_value = ".", help = "Directory to show")]
        dir: String,
        #[arg(long, default_value_t = 2, help = "Maximum depth")]
        depth: usize,
        #[arg(long, help = "Include hidden files and folders")]
        hidden: bool,
    },
    #[command(about = "Docker container manager and safe pruning")]
    Docker {
        #[command(subcommand)]
        action: commands::docker::DockerAction,
    },
    #[command(
        name = "clean-cache",
        about = "Scan and interactively clean build & package caches"
    )]
    CleanCache {
        #[arg(long, help = "Send the scan to the panel webserver")]
        serve: bool,
        #[arg(long, default_value_t = crate::panel::default_port(), help = "Panel port")]
        port: u16,
    },
    #[command(
        name = "audit-deps",
        about = "Scan lockfiles & system packages for known vulnerabilities"
    )]
    AuditDeps {
        #[arg(
            value_name = "DIR",
            help = "Directory to scan (interactively prompted if omitted)"
        )]
        dir: Option<String>,
        #[arg(long, help = "Skip interactive prompts and audit everything")]
        no_prompt: bool,
        #[arg(long, help = "Never ask to open advisories in the browser")]
        no_open: bool,
        #[arg(
            long,
            value_name = "LEVEL",
            help = "Minimum severity to report: low, moderate, high, critical, all"
        )]
        min_severity: Option<String>,
        #[arg(
            long,
            value_name = "CATS",
            help = "Only report categories (comma list): infected, exploited, unmaintained, vulnerable"
        )]
        category: Option<String>,
    },
    #[command(
        name = "cert-check",
        about = "Inspect the TLS certificate of a remote server"
    )]
    CertCheck {
        #[arg(required = true, value_name = "DOMAIN")]
        domain: String,
    },
    #[command(
        name = "dns-lookup",
        about = "Query A, AAAA, MX, TXT, CNAME, and NS records at once"
    )]
    DnsLookup {
        #[arg(required = true, value_name = "DOMAIN")]
        domain: String,
    },
    #[command(
        name = "local-s3",
        about = "Spin up an ephemeral local S3-compatible server (MinIO)"
    )]
    LocalS3,
    #[command(
        name = "port-forward",
        about = "SSH port forwarding with auto-retry and health monitoring"
    )]
    PortForward {
        #[arg(
            required = true,
            value_name = "LOCAL:user@host:REMOTE",
            help = "e.g. 8080:user@host:5432"
        )]
        spec: String,
        #[arg(long, default_value_t = 3, help = "Reconnect attempts (0 = unlimited)")]
        retries: usize,
        #[arg(
            long,
            default_value_t = 5,
            help = "Health check interval in seconds"
        )]
        interval: u64,
    },
    #[command(
        name = "readme-init",
        about = "Generate a README.md for a new project"
    )]
    ReadmeInit {
        #[arg(value_name = "NAME", help = "Project name")]
        name: Option<String>,
        #[arg(long, value_name = "DESC", help = "Short description")]
        desc: Option<String>,
        #[arg(long, value_name = "LICENSE", help = "License (default: MIT)")]
        license: Option<String>,
    },
    #[command(about = "Pomodoro focus timer with work/break intervals")]
    Focus {
        #[arg(long, default_value_t = 25, help = "Work minutes")]
        work: u64,
        #[arg(long = "short", default_value_t = 5, help = "Short break minutes")]
        short_break: u64,
        #[arg(long, default_value_t = 15, help = "Long break minutes")]
        long_break: u64,
        #[arg(long, default_value_t = 4, help = "Number of cycles")]
        cycles: u64,
    },
    #[command(about = "Measure internet download speed")]
    Speedtest,
    #[command(
        name = "search-docs",
        about = "Search documentation via cheat.sh or tldr"
    )]
    SearchDocs {
        #[arg(value_name = "QUERY", help = "Search query")]
        query: String,
        #[arg(long = "source", value_name = "SRC", help = "cheat, tldr")]
        source: Option<String>,
    },
    #[command(about = "Read a file with syntax-highlighted output")]
    Reader {
        #[arg(value_name = "FILE", help = "Path to file")]
        file: String,
    },
    #[command(about = "Record a terminal session (asciinema wrapper)")]
    Asciicast {
        #[arg(short = 'o', long, value_name = "OUTPUT", help = "Output file")]
        output: Option<String>,
        #[arg(
            value_name = "CMD",
            num_args = 0..,
            trailing_var_arg = true,
            help = "Command to record (default: shell)"
        )]
        cmd: Vec<String>,
    },
    #[command(about = "Generate a QR code in the terminal (and optionally PNG)")]
    Qr {
        #[arg(value_name = "TEXT", help = "Text to encode")]
        text: String,
        #[arg(short = 'o', long, value_name = "OUTPUT", help = "Save as PNG file")]
        out: Option<String>,
    },
    #[command(
        name = "render-md",
        about = "Render Markdown files directly in the terminal"
    )]
    RenderMd {
        #[arg(value_name = "FILE", help = "Markdown file path")]
        file: String,
    },
    #[command(
        name = "color-palette",
        about = "Display ANSI color palette in the terminal"
    )]
    ColorPalette,
    #[command(about = "Simple local todo list manager (add, list, done, remove)")]
    Todo {
        #[arg(value_name = "ACTION", help = "add, list, done, or remove")]
        action: String,
        #[arg(
            value_name = "TEXT",
            num_args = 0..,
            help = "Task text (for add) or ID (for done/remove)"
        )]
        text: Vec<String>,
        #[arg(short = 'i', long, default_value_t = 0, help = "Task ID (for done/remove)")]
        id: usize,
    },
    #[command(
        name = "gen-pass",
        about = "Generate secure random passwords"
    )]
    GenPass {
        #[arg(short = 'l', long, default_value_t = 20, help = "Password length")]
        length: usize,
        #[arg(long, help = "Exclude symbols")]
        no_symbols: bool,
        #[arg(long, help = "Exclude numbers")]
        no_numbers: bool,
        #[arg(short = 'n', long, default_value_t = 1, help = "Number of passwords")]
        count: usize,
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
        Some(Commands::Help { command }) => match command {
            Some(cmd) => commands::help::run(&commands::help::HelpAction::For { command: cmd }),
            None => commands::help::run(&commands::help::HelpAction::All),
        },
        Some(Commands::ShareSession { action }) => commands::share::run(&action),
        Some(Commands::Alias { action }) => commands::alias::run(&action),
        Some(Commands::System) => commands::system::run(),
        Some(Commands::Pkg { action }) => commands::pkg::run(&action),
        Some(Commands::Git { action }) => commands::git::run(&action),
        Some(Commands::Setup) => commands::setup::run(),
        Some(Commands::Mc { action }) => commands::mc::run(&action),
        Some(Commands::Status { action }) => commands::status::run(&action),
        Some(Commands::Discord { action }) => commands::discord::run(&action),
        Some(Commands::Convert { input, to }) => {
            commands::convert::run(&commands::convert::ConvertAction::Run { input, to })
        }
        Some(Commands::Encrypt { action }) => commands::encrypt::run(&action),
        Some(Commands::App { action }) => commands::app::run(&action),
        Some(Commands::Ai { action }) => commands::ai::run(&action),
        Some(Commands::CopyCtx) => commands::copyctx::run(),
        Some(Commands::Memo { action }) => commands::memo::run(&action),
        Some(Commands::Secret { action }) => commands::secret::run(&action),
        Some(Commands::Webhook { action }) => commands::webhook::run(&action),
        Some(Commands::PrPrep {
            skip_tests,
            skip_lint,
            skip_fmt,
            no_open,
        }) => commands::pr::run(&commands::pr::PrAction::Prep {
            skip_tests,
            skip_lint,
            skip_fmt,
            no_open,
        }),
        Some(Commands::PrCheckout { target }) => {
            commands::pr::run(&commands::pr::PrAction::Checkout { target })
        }
        Some(Commands::GitWhoBroke { command }) => commands::git_extras::who_broke(&command),
        Some(Commands::GitImpact) => commands::git_extras::impact(),
        Some(Commands::GitCatchup) => commands::git_extras::catchup(),
        Some(Commands::Share { file }) => commands::upload::run(&file),
        Some(Commands::Dedupe { dir }) => commands::dedupe::run(&dir),
        Some(Commands::Media { action }) => commands::media::run(&action),
        Some(Commands::Download { action }) => commands::download::run(&action),
        Some(Commands::Battery { serve, interval, port }) => {
            commands::battery::run(serve, interval, port)
        }
        Some(Commands::KillHeavy {
            cpu,
            mem,
            all,
            serve,
            port,
        }) => commands::killheavy::run(cpu, mem, all, serve, port),
        Some(Commands::Manage { action }) => commands::manage::run(&action),
        Some(Commands::Ports { serve, port }) => commands::ports::run(serve, port),
        Some(Commands::TreeView { dir, depth, hidden }) => {
            commands::treeview::run(&dir, depth, hidden)
        }
        Some(Commands::Docker { action }) => commands::docker::run(&action),
        Some(Commands::CleanCache { serve, port }) => {
            commands::cleancache::run(serve, port)
        }
        Some(Commands::AuditDeps {
            dir,
            no_prompt,
            no_open,
            min_severity,
            category,
        }) => commands::auditdeps::run(dir, no_prompt, no_open, min_severity, category),
        Some(Commands::CertCheck { domain }) => commands::cert::run(&domain),
        Some(Commands::DnsLookup { domain }) => commands::dns::run(&domain),
        Some(Commands::LocalS3) => commands::locals3::run(),
        Some(Commands::PortForward {
            spec,
            retries,
            interval,
        }) => commands::portfwd::run(&spec, retries, interval),
        Some(Commands::ReadmeInit { name, desc, license }) => {
            commands::readme::run(name, desc, license)
        }
        Some(Commands::Focus {
            work,
            short_break,
            long_break,
            cycles,
        }) => commands::focus::run(work, short_break, long_break, cycles),
        Some(Commands::Speedtest) => commands::speedtest::run(),
        Some(Commands::SearchDocs { query, source }) => {
            commands::searchdocs::run(query, source)
        }
        Some(Commands::Reader { file }) => commands::reader::run(file),
        Some(Commands::Asciicast { output, cmd }) => commands::asciicast::run(output, cmd),
        Some(Commands::Qr { text, out }) => commands::qr::run(text, out),
        Some(Commands::RenderMd { file }) => commands::rendermd::run(file),
        Some(Commands::ColorPalette) => commands::colorpalette::run(),
        Some(Commands::Todo { action, text, id }) => commands::todo::run(&action, text, id),
        Some(Commands::GenPass {
            length,
            no_symbols,
            no_numbers,
            count,
        }) => commands::genpass::run(length, no_symbols, no_numbers, count),
        None => {
            commands::help::run(&commands::help::HelpAction::All);
        }
    }
}

fn print_version() {
    use crate::style;
    use owo_colors::OwoColorize;

    println!("{}", style::proto_banner());
    println!(
        "{} {}",
        "proto".style(style::Theme::HEADER).bold(),
        env!("CARGO_PKG_VERSION").style(style::Theme::MUTED)
    );
    println!(
        "{}",
        "Your friendly protogen CLI companion".style(style::Theme::MUTED)
    );
}

fn print_short_help() {
    commands::help::run(&commands::help::HelpAction::All);
}
