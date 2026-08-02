use crate::style;
use clap::Subcommand;
use owo_colors::OwoColorize;

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
    println!(
        "{}  {}",
        "◆".style(style::Theme::ACCENT),
        "Your friendly protogen CLI companion\n".style(style::Theme::MUTED)
    );

    println!("{}", "USAGE:".style(style::Theme::HEADER));
    println!(
        "  {} <command> [options]\n",
        "proto".style(style::Theme::ACCENT)
    );

    println!("{}", "COMMANDS:".style(style::Theme::HEADER));
    print_cmd(
        "help",
        "[command]",
        "Show this help or help for a specific command",
    );
    print_cmd("system", "", "Display beautiful system information");
    print_cmd(
        "alias",
        "create|list|remove",
        "Interactive shell alias builder",
    );
    print_cmd(
        "share-session",
        "create|join",
        "Share terminal via tmate/tmux",
    );
    print_cmd("pkg", "<action>", "Cross-distro package manager wrapper");
    print_cmd("git", "<action>", "Git workflow enhancements");
    print_cmd("setup", "", "Interactive configuration wizard");
    print_cmd(
        "mc",
        "resource_pack|server",
        "Minecraft resource packs & servers",
    );
    print_cmd(
        "status",
        "ping|monitor|serve|report",
        "Network monitoring tools",
    );
    print_cmd("discord", "bot|quest", "Discord bot creator & tools");
    print_cmd(
        "app",
        "doctor|port|nuke|snap",
        "Project diagnostics & cleanup",
    );
    print_cmd(
        "ai",
        "chat|setup|summarize|explain",
        "AI assistant & changelog generator",
    );
    print_cmd(
        "copy-ctx",
        "",
        "Bundle repo into LLM-ready clipboard context",
    );
    print_cmd("memo", "add|list|clear", "Location-aware scratchpad notes");
    print_cmd("secret", "mask", "Scan history/logs for leaked secrets");
    print_cmd("webhook", "listen", "Print webhooks as formatted JSON");
    print_cmd("pr-prep", "", "Run tests, lint, format, open PR page");
    print_cmd(
        "pr-checkout",
        "<PR#|URL>",
        "Check out a PR into a temp branch",
    );
    print_cmd(
        "git-who-broke",
        "[test-cmd]",
        "Bisect to find the breaking commit",
    );
    print_cmd("git-impact", "", "Branch risk score before merging");
    print_cmd("git-catchup", "", "What changed upstream since your pull");
    print_cmd("share", "<FILE>", "Upload file to a temporary host");
    print_cmd("dedupe", "[DIR]", "Find and remove exact duplicates");
    print_cmd("media", "shrink", "Compress images/videos in place");
    print_cmd("cert-check", "<DOMAIN>", "Inspect a remote TLS certificate");
    print_cmd("dns-lookup", "<DOMAIN>", "Query all DNS records at once");
    print_cmd("local-s3", "", "Spin up an ephemeral MinIO server");
    print_cmd(
        "port-forward",
        "<LOCAL:HOST:REMOTE>",
        "SSH forwarding with auto-retry",
    );
    print_cmd(
        "download",
        "video|music",
        "Download videos & music via yt-dlp",
    );
    print_cmd(
        "battery",
        "[--serve]",
        "Laptop battery health & live wattage",
    );
    print_cmd(
        "kill-heavy",
        "[--cpu N] [--mem MB]",
        "Find & kill high CPU/RAM processes",
    );
    print_cmd(
        "ports",
        "[--serve]",
        "Interactive listening-ports dashboard",
    );
    print_cmd(
        "tree-view",
        "[--depth N] [DIR]",
        "ASCII folder tree that respects .gitignore",
    );
    print_cmd(
        "docker",
        "containers|prune-safe",
        "Docker container manager & safe pruning",
    );
    print_cmd(
        "clean-cache",
        "[--serve]",
        "Scan & clean package, build & docker caches",
    );
    print_cmd(
        "audit-deps",
        "[DIR]",
        "Scan lockfiles & system packages for known vulnerabilities",
    );

    println!("\n{}", "FLAGS:".style(style::Theme::HEADER));
    print_cmd("--version", "", "Print version and exit");
    print_cmd("--help", "", "Print help information");

    println!(
        "\n{} {}\n",
        "Run".style(style::Theme::MUTED),
        "'proto help <command>' for more info.".style(style::Theme::MUTED)
    );
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
            println!(
                "\n{} Templates include full runnable code, env config, and deps.",
                "  ".dimmed()
            );
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
        "secret" => {
            println!("{}", "proto secret".style(style::Theme::HEADER));
            println!("  Scan shell history and log files for leaked credentials.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto secret mask                      Scan shell histories (default)");
            println!("  proto secret mask --file <PATH>        Scan a specific file/directory");
            println!("  proto secret mask --dry-run            Alert only, don't rewrite files\n");
            println!("{}", "DETECTS:".style(style::Theme::HEADER));
            println!("  AWS keys, GitHub tokens, OpenAI keys, Google API keys, Slack tokens,");
            println!("  Stripe keys, JWTs, private keys, and generic secret assignments.");
        }
        "webhook" => {
            println!("{}", "proto webhook".style(style::Theme::HEADER));
            println!("  Listen for webhooks and print them as formatted JSON.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto webhook listen [PORT]    Listen on a port (default: 9000)");
            println!("  proto webhook listen 8080 --no-tunnel   Skip the ngrok public URL\n");
            println!("{}", "FEATURES:".style(style::Theme::HEADER));
            println!("  Auto-opens an ngrok HTTP tunnel when available");
            println!("  Colorized JSON output + request headers");
        }
        "pr-prep" => {
            println!("{}", "proto pr-prep".style(style::Theme::HEADER));
            println!("  One-command PR readiness check.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto pr-prep\n");
            println!("{}", "STEPS:".style(style::Theme::HEADER));
            println!("  1. Auto-format code (cargo fmt / prettier / ruff / gofmt)");
            println!("  2. Run linter (clippy / lint script / ruff / go vet)");
            println!("  3. Run tests (cargo test / npm test / pytest / go test)");
            println!("  4. Scan changed files for console.log / print / TODO");
            println!("  5. Open the PR create page in your browser\n");
            println!("{}", "FLAGS:".style(style::Theme::HEADER));
            println!("  --skip-tests  --skip-lint  --skip-fmt  --no-open");
        }
        "pr-checkout" => {
            println!("{}", "proto pr-checkout".style(style::Theme::HEADER));
            println!("  Check out a pull request locally into a temp branch.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto pr-checkout 42");
            println!("  proto pr-checkout https://github.com/owner/repo/pull/42");
            println!("  proto pr-checkout https://gitlab.com/owner/repo/-/merge_requests/42");
            println!("  proto pr-checkout owner/repo#42\n");
            println!("{}", "NOTES:".style(style::Theme::HEADER));
            println!("  Creates branch pr-<N>; clones the repo first if not inside one.");
        }
        "git-who-broke" => {
            println!("{}", "proto git-who-broke".style(style::Theme::HEADER));
            println!("  Auto-bisect to pinpoint the commit that broke your tests.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!(
                "  proto git-who-broke                    (auto: cargo test / npm test / pytest)"
            );
            println!("  proto git-who-broke cargo test -- --ignored\n");
            println!("{}", "FLOW:".style(style::Theme::HEADER));
            println!("  Stashes changes, finds a good commit, runs git bisect,");
            println!("  then restores your branch and stash automatically.");
        }
        "git-impact" => {
            println!("{}", "proto git-impact".style(style::Theme::HEADER));
            println!("  Calculate branch risk before merging.\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  Files changed with +adds/-dels, per-file risk tags,");
            println!("  line churn, and a 0-100 blast-radius score with verdict.");
        }
        "git-catchup" => {
            println!("{}", "proto git-catchup".style(style::Theme::HEADER));
            println!("  Show everything that changed upstream since your last pull.\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  Commits behind, files changed, docs updated, and merged PRs");
            println!("  (via gh) on the default branch.");
        }
        "share" => {
            println!("{}", "proto share".style(style::Theme::HEADER));
            println!("  Upload a file to a temporary, self-expiring host.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto share <FILE>\n");
            println!("{}", "FEATURES:".style(style::Theme::HEADER));
            println!("  Asks for confirmation before uploading");
            println!("  Renders image previews in kitty/other image terminals");
            println!("  Copies the download URL to your clipboard");
            println!("  Falls back across bashupload.com, file.io, tmpfiles.org\n");
            println!("{}", "NOTES:".style(style::Theme::HEADER));
            println!("  Files expire after a few days or a limited number of downloads.");
            println!("  Treat links as ephemeral — don't share sensitive data.");
        }
        "dedupe" => {
            println!("{}", "proto dedupe".style(style::Theme::HEADER));
            println!("  Find exact duplicate files by content hash (not just name).\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto dedupe [DIR]     Scan a directory (default: current)\n");
            println!("{}", "FLOW:".style(style::Theme::HEADER));
            println!("  1. Groups files by size, then SHA-256 hashes candidates");
            println!("  2. For each group, choose: skip, delete, or symlink duplicates");
            println!("  3. Symlinks point to the kept file (relative paths, portable)");
        }
        "media" => {
            println!("{}", "proto media".style(style::Theme::HEADER));
            println!("  Compress media files in place using system tools.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto media shrink [FILE|DIR]   Default: current directory\n");
            println!("{}", "FORMATS & TOOLS:".style(style::Theme::HEADER));
            println!("  png   → optipng / pngcrush (lossless)");
            println!("  jpg   → jpegoptim --strip-all (lossless)");
            println!("  webp  → cwebp -lossless");
            println!("  video → ffmpeg (h264 crf 18, keeps audio)");
            println!("  Files are only replaced when the result is smaller.");
        }
        "cert-check" => {
            println!("{}", "proto cert-check".style(style::Theme::HEADER));
            println!("  Inspect the TLS certificate of a remote server.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto cert-check example.com\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  Subject, issuer, validity window, days remaining,");
            println!("  and SAN domains. Warns when expiry is near.");
            println!("  Uses openssl s_client under the hood (port 443).");
        }
        "dns-lookup" => {
            println!("{}", "proto dns-lookup".style(style::Theme::HEADER));
            println!("  One-shot DNS diagnostic.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto dns-lookup example.com\n");
            println!("{}", "QUERIES:".style(style::Theme::HEADER));
            println!("  A, AAAA, CNAME, MX, TXT, and NS records in one table.");
            println!("  Requires dig (bind-tools).");
        }
        "local-s3" => {
            println!("{}", "proto local-s3".style(style::Theme::HEADER));
            println!("  Spin up an ephemeral local S3-compatible server.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto local-s3\n");
            println!("{}", "ENGINE:".style(style::Theme::HEADER));
            println!("  Uses a minio binary or Docker (minio/minio) automatically.");
            println!("  Data lives in ./.proto-s3-data; Ctrl+C stops the server.");
            println!("  Prints ready-to-use aws cli and mc commands.");
        }
        "port-forward" => {
            println!("{}", "proto port-forward".style(style::Theme::HEADER));
            println!("  SSH port forwarding wrapper with auto-retry.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto port-forward 8080:user@host:5432");
            println!("  proto port-forward 3000:db.example.com:5432\n");
            println!("{}", "OPTIONS:".style(style::Theme::HEADER));
            println!("  --retries N      Reconnect attempts (default: 3, 0 = unlimited)");
            println!("  --interval SECS  Health check interval (default: 5)\n");
            println!("{}", "FEATURES:".style(style::Theme::HEADER));
            println!("  Auto-reconnects dropped connections, monitors the local");
            println!("  port and shows UP/DOWN status changes.");
        }
        "download" => {
            println!("{}", "proto download".style(style::Theme::HEADER));
            println!("  Download videos and music via yt-dlp.\n");
            println!("{}", "VIDEO:".style(style::Theme::HEADER));
            println!("  proto download video [URL]             Interactive (prompts URL if missing)");
            println!("  proto download video <URL> --format 1080p --dir ~/Videos");
            println!("  proto download video <URL> --format audio-mp3 --subtitles\n");
            println!("  FORMATS: best | 1080p | 720p | 480p | audio-mp3\n");
            println!("{}", "MUSIC:".style(style::Theme::HEADER));
            println!("  proto download music <URL>             YouTube or SoundCloud playlist");
            println!("  proto download music <URL> --amount 10 --browser firefox\n");
            println!("{}", "SOUNDCLOUD:".style(style::Theme::HEADER));
            println!("  Tracks go to <dir>/<uploader>/<title>.mp3 (256k + metadata/cover).");
            println!("  download_log.txt tracks already-downloaded songs; failures are");
            println!("  written to download_log_ERROR.txt. Runs >15 min are skipped.");
            println!("  Cookies come from an installed browser (auto-detected,");
            println!("  override with --browser chrome|firefox|etc).\n");
            println!("  proto download music https://soundcloud.com/artist --artist");
            println!("      Downloads an artist's whole catalog (profile page).\n");
            println!("{}", "YOUTUBE:".style(style::Theme::HEADER));
            println!("  mp3 256k per-uploader folders. Options: --amount N, --newest,");
            println!("  --yes (skip confirmations), --dir <path>.");
        }
        "battery" => {
            println!("{}", "proto battery".style(style::Theme::HEADER));
            println!("  Laptop battery health diagnostics.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto battery                       Show health snapshot");
            println!("  proto battery --serve               Live-stream to the panel");
            println!("  proto battery --serve --interval 2  Poll every 2s\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  Charge %, cycle count, current vs design capacity,");
            println!("  health %, and live draw in watts (when discharging).");
            println!("  Reads /sys/class/power_supply/BAT* (requires a battery).");
        }
        "kill-heavy" => {
            println!("{}", "proto kill-heavy".style(style::Theme::HEADER));
            println!("  Find and interactively kill heavy processes.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto kill-heavy                    Flag >10% CPU or >512MB RSS");
            println!("  proto kill-heavy --cpu 5 --mem 256  Lower thresholds");
            println!("  proto kill-heavy --all              Show every process");
            println!("  proto kill-heavy --serve            Send scan to the panel\n");
            println!("{}", "FLOW:".style(style::Theme::HEADER));
            println!("  1. Lists top 15 heavy processes (multi-select)");
            println!("  2. Sends SIGTERM, escalates to SIGKILL after ~1s");
        }
        "ports" => {
            println!("{}", "proto ports".style(style::Theme::HEADER));
            println!("  Interactive listening-ports dashboard.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto ports                         Terminal dashboard");
            println!("  proto ports --serve                 Stream to the panel\n");
            println!("{}", "FEATURES:".style(style::Theme::HEADER));
            println!("  Parses `ss -tulpnH`; shows proto, port, pid, process,");
            println!("  and address. Select an entry to kill it (after confirm).");
            println!("  Requires iproute2 (ss).");
        }
        "tree-view" => {
            println!("{}", "proto tree-view".style(style::Theme::HEADER));
            println!("  ASCII folder tree that respects .gitignore.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto tree-view                     Current dir, depth 2");
            println!("  proto tree-view src --depth 3       Deeper tree");
            println!("  proto tree-view --hidden            Include hidden files\n");
            println!("{}", "FEATURES:".style(style::Theme::HEADER));
            println!("  Respects .gitignore rules (incl. !negations and dir-only),");
            println!("  ignores .git and target/ by default, sorts dirs first.");
        }
        "docker" => {
            println!("{}", "proto docker".style(style::Theme::HEADER));
            println!("  Docker container manager & safe pruning.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto docker containers              Interactive manager");
            println!("  proto docker containers --serve      Stream to the panel");
            println!("  proto docker prune-safe              Remove dangling objects\n");
            println!("{}", "CONTAINERS:".style(style::Theme::HEADER));
            println!("  List running (or --all) containers; start/stop/restart,");
            println!("  logs, inspect, or remove the selected one.\n");
            println!("{}", "PRUNE-SAFE:".style(style::Theme::HEADER));
            println!("  Removes dangling images, stopped containers, and unused");
            println!("  volumes — but keeps anything matching the current git branch.");
            println!("  Shows `docker system df` BEFORE and AFTER, with a confirm.");
        }
        "clean-cache" => {
            println!("{}", "proto clean-cache".style(style::Theme::HEADER));
            println!("  Scan and interactively clean build & package caches.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto clean-cache                   Interactive scan & clean");
            println!("  proto clean-cache --serve           Send scan to the panel\n");
            println!("{}", "SCANS:".style(style::Theme::HEADER));
            println!("  npm, pip, uv, bun, yarn, cargo registry, cargo src, go build,");
            println!("  gradle, yay (AUR build), paru (AUR build), pacman (sudo),");
            println!("  apt (sudo), dnf (sudo), and docker builder cache.\n");
            println!("{}", "SHOWS:".style(style::Theme::HEADER));
            println!("  Disk free before and after cleaning + bytes recovered.");
            println!("  Always asks for confirmation before deleting anything.");
        }
        "audit-deps" => {
            println!("{}", "proto audit-deps".style(style::Theme::HEADER));
            println!("  Scan lockfiles & system packages for known vulnerabilities.\n");
            println!("{}", "USAGE:".style(style::Theme::HEADER));
            println!("  proto audit-deps                     Interactive (prompts for dir/sources)");
            println!("  proto audit-deps /path/to/project    Scan another directory");
            println!("  proto audit-deps --no-prompt         Audit everything non-interactively");
            println!("  proto audit-deps --min-severity high Only report High+ findings");
            println!("  proto audit-deps --category infected  Only show infected/malware findings\n");
            println!("{}", "FLAGS:".style(style::Theme::HEADER));
            println!("  --no-prompt         Skip prompts (dir default ., all sources, all severities)");
            println!("  --no-open           Never ask to open advisories in the browser");
            println!("  --min-severity LVL  Filter: all, low, moderate, high, critical");
            println!("  --category CATS     Only report categories: infected, exploited, unmaintained, cve\n");
            println!("{}", "LOCKFILES:".style(style::Theme::HEADER));
            println!("  package-lock.json, yarn.lock, pnpm-lock.yaml (npm)");
            println!("  Cargo.lock (crates.io), go.sum (Go), pom.xml (Maven)");
            println!("  packages.lock.json (NuGet), pubspec.lock (Pub)");
            println!("  mix.lock (Hex), Package.resolved (Swift)");
            println!("  requirements.txt, Pipfile.lock, poetry.lock (PyPI)");
            println!("  Gemfile.lock (RubyGems), composer.lock (Packagist)");
            println!("  conan.lock (Conan).\n");
            println!("{}", "SYSTEM:".style(style::Theme::HEADER));
            println!("  Arch: pacman/AUR checks against Arch Security Tracker (ASA/AVG).");
            println!("  Debian/Ubuntu: dpkg/apt checks via OSV Debian/Ubuntu databases.");
            println!("  Automatically detected by package manager.\n");
            println!("{}", "DATA:".style(style::Theme::HEADER));
            println!("  Categories: [INFECTED] malware/supply-chain, [EXPLOITED] actively");
            println!("  exploited, [UNMAINTAINED] abandoned/end-of-life packages.");
            println!("  A HIGH RISK summary at the end lists infected/exploited/critical");
            println!("  packages and advisory pages can be opened in the browser.");
            println!("  Requires a network connection. Scans up to 8 directories deep");
            println!("  and skips node_modules, target/, .git, vendor, dist, build.");
        }
        other => {
            println!(
                "{} Unknown command: '{}'",
                style::error(""),
                other.style(style::Theme::ACCENT)
            );
            println!(
                "Run {} to see all available commands.",
                "proto help".style(style::Theme::ACCENT)
            );
        }
    }
}

fn print_cmd(name: &str, args: &str, desc: &str) {
    let name_part = if args.is_empty() {
        format!("  {}          ", name.style(style::Theme::ACCENT))
    } else {
        format!(
            "  {} {}",
            name.style(style::Theme::ACCENT),
            args.style(style::Theme::BOLD)
        )
    };
    println!(
        "{}{}",
        format!("{:40}", name_part),
        desc.style(style::Theme::MUTED)
    );
}
