use clap::Subcommand;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum BuildAction {
    #[command(about = "Build a portable package installer")]
    Pack {
        #[command(subcommand)]
        action: PackAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum PackAction {
    #[command(about = "Create a new pack configuration interactively")]
    Create,
    #[command(about = "Edit an existing pack configuration")]
    Edit,
    #[command(about = "Build the portable installer executable")]
    Build,
    #[command(about = "Test/dry-run the pack configuration")]
    Test,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PackConfig {
    name: String,
    version: String,
    description: String,
    author: String,
    lock_os: Option<String>,
    packages: Vec<OsPackages>,
    post_install: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OsPackages {
    os: String,
    packages: Vec<String>,
}

pub fn run(action: &BuildAction) {
    match action {
        BuildAction::Pack { action } => pack(action),
    }
}

fn pack(action: &PackAction) {
    match action {
        PackAction::Create => create(),
        PackAction::Edit => edit(),
        PackAction::Build => build(),
        PackAction::Test => test(),
    }
}

fn create() {
    use dialoguer::{Confirm, Input, MultiSelect};

    println!("{}", style::proto_banner());
    println!("{}\n", "Pack Creator".style(style::Theme::HEADER));

    let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Pack name")
        .default("my-app".into())
        .interact_text().unwrap();

    let folder = name.replace(' ', "-").to_lowercase();
    let path = std::path::PathBuf::from(&folder);

    if path.exists() {
        eprintln!("{} '{}' already exists.", style::error(""), folder);
        return;
    }
    std::fs::create_dir_all(&path).unwrap();

    let version: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Version")
        .default("1.0.0".into())
        .interact_text().unwrap();

    let desc: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Description")
        .default("A portable app installer".into())
        .interact_text().unwrap();

    let author: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Author")
        .default(whoami().unwrap_or_else(|| "unknown".into()))
        .interact_text().unwrap();

    let lock = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Lock to current OS? (disables other OS support)")
        .default(false).interact().unwrap_or(false);

    let lock_os = if lock {
        Some(detect_os())
    } else {
        None
    };

    println!("\n{}", "Select packages per OS:".style(style::Theme::HEADER));

    let os_list = if let Some(ref os) = lock_os {
        vec![os.clone()]
    } else {
        vec!["linux".into(), "macos".into(), "windows".into()]
    };

    let mut all_packages = Vec::new();

    for os_name in &os_list {
        println!("\n  {} {}:", "◆".style(style::Theme::ACCENT), os_name.style(style::Theme::ACCENT));
        let pkgs: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(&format!("Packages for {} (space-separated)", os_name))
            .allow_empty(true)
            .interact_text().unwrap();

        if !pkgs.trim().is_empty() {
            let pkg_list: Vec<String> = pkgs.split_whitespace().map(|s| s.to_string()).collect();
            all_packages.push(OsPackages { os: os_name.clone(), packages: pkg_list });
        }
    }

    let post: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Post-install commands (semicolon-separated)")
        .allow_empty(true)
        .interact_text().unwrap();

    let post_install: Vec<String> = post.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    let config = PackConfig {
        name,
        version,
        description: desc,
        author,
        lock_os,
        packages: all_packages,
        post_install,
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(path.join("pkg.json"), &json).unwrap();

    println!("\n{} {}", style::success(""), format!("Pack '{}' created!", folder).style(style::Theme::ACCENT));
    println!("  cd {} && proto pkg build pack build", folder);
}

fn edit() {
    let cwd = std::env::current_dir().unwrap_or_default();
    let cfg_path = cwd.join("pkg.json");

    if !cfg_path.exists() {
        eprintln!("{} No pkg.json found in current directory.", style::error(""));
        eprintln!("{} Create one with: {}", style::warn(""), "proto pkg build pack create".style(style::Theme::ACCENT));
        return;
    }

    let json = std::fs::read_to_string(&cfg_path).unwrap();
    let mut config: PackConfig = serde_json::from_str(&json).unwrap();

    use dialoguer::{Confirm, Input};

    println!("{} Editing: {}", "◆".style(style::Theme::ACCENT), config.name.style(style::Theme::ACCENT));
    println!("{}", style::divider());

    let new_desc: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Description")
        .default(config.description.clone())
        .interact_text().unwrap();
    config.description = new_desc;

    let new_ver: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Version")
        .default(config.version.clone())
        .interact_text().unwrap();
    config.version = new_ver;

    if let Some(ref os) = config.lock_os {
        println!("  {} Locked to {}", "🔒".dimmed(), os.style(style::Theme::ACCENT));
    }

    for os_pkg in &mut config.packages {
        let current = os_pkg.packages.join(" ");
        let updated: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(&format!("Packages for {} (space-separated)", os_pkg.os))
            .default(current)
            .interact_text().unwrap();
        os_pkg.packages = updated.split_whitespace().map(|s| s.to_string()).collect();
    }

    let json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(&cfg_path, &json).unwrap();
    println!("\n{} Config updated.", style::success(""));
}

fn build() {
    let cwd = std::env::current_dir().unwrap_or_default();
    let cfg_path = cwd.join("pkg.json");

    if !cfg_path.exists() {
        eprintln!("{} No pkg.json found.", style::error(""));
        return;
    }

    let json = std::fs::read_to_string(&cfg_path).unwrap();
    let config: PackConfig = match serde_json::from_str(&json) {
        Ok(c) => c,
        Err(e) => { eprintln!("{} Invalid pkg.json: {}", style::error(""), e); return; }
    };

    let sp = style::Spinner::new(&format!("Building '{}'...", config.name));

    let mut needed: Vec<String> = Vec::new();
    let os = if let Some(ref lock) = config.lock_os { lock.clone() } else { detect_os() };

    for os_pkg in &config.packages {
        let matches = os_pkg.os == os || os_pkg.os == "linux" || os_pkg.os == "any";
        if matches {
            needed.extend(os_pkg.packages.clone());
        }
    }

    if needed.is_empty() {
        sp.fail("No packages configured for this OS.");
        return;
    }

    let config_b64 = base64_encode(&json);
    let installer_name = format!("{}-installer.sh", config.name.to_lowercase().replace(' ', "-"));
    let installer_path = cwd.join(&installer_name);

    let script = generate_installer(&config, &config_b64, &needed);
    std::fs::write(&installer_path, &script).unwrap();
    std::fs::set_permissions(&installer_path, {
        use std::os::unix::fs::PermissionsExt;
        std::fs::Permissions::from_mode(0o755)
    }).unwrap();

    sp.done(&format!("Built {}", installer_name));
    println!("\n{} {}", style::success(""), format!("Portable installer ready:").style(style::Theme::ACCENT));
    println!("  ./{}", installer_name);
    println!("\n  Run it on any {} system to install:", os.style(style::Theme::ACCENT));
    for pkg in &needed {
        println!("    {} {}", "▸".style(style::Theme::ACCENT), pkg.style(style::Theme::MUTED));
    }
}

fn test() {
    let cwd = std::env::current_dir().unwrap_or_default();
    let cfg_path = cwd.join("pkg.json");

    if !cfg_path.exists() {
        eprintln!("{} No pkg.json found.", style::error(""));
        return;
    }

    let json = std::fs::read_to_string(&cfg_path).unwrap();
    let config: PackConfig = match serde_json::from_str(&json) {
        Ok(c) => c,
        Err(e) => { eprintln!("{} Invalid: {}", style::error(""), e); return; }
    };

    println!("{} Dry-run: {}", "◆".style(style::Theme::ACCENT), config.name.style(style::Theme::ACCENT));
    println!("{}", style::divider());

    let os = if let Some(ref lock) = config.lock_os { lock.clone() } else { detect_os() };
    println!("{}", style::label_value("OS", &os));

    let mut found = false;
    for os_pkg in &config.packages {
        let matches = os_pkg.os == os || os_pkg.os == "linux" || os_pkg.os == "any";
        if matches {
            found = true;
            println!("{}", style::label_value("Packages", &os_pkg.packages.join(", ")));
        }
    }

    if !found {
        println!("{} No packages for this OS.", style::warn(""));
    }

    if !config.post_install.is_empty() {
        println!("{}", style::label_value("Post-install", &config.post_install.join("; ")));
    }

    println!("{}", style::divider());
    let lock_str = config.lock_os.clone().unwrap_or_else(|| "any".into());
    println!("\n{}", style::label_value("Locked to", &lock_str));
    println!("{} OS matches config ✓", style::success(""));
}

fn generate_installer(config: &PackConfig, config_b64: &str, packages: &[String]) -> String {
    let pkg_list = packages.join(" ");
    format!(r##"#!/usr/bin/env bash
set -euo pipefail

BOLD="\033[1m" CYAN="\033[0;36m" GREEN="\033[0;32m" RED="\033[0;31m" YELLOW="\033[1;33m" NC="\033[0m"
info(){{ echo -e "${{CYAN}}  ◆${{NC}} $1"; }}
success(){{ echo -e "${{GREEN}}  ✔${{NC}} $1"; }}
err(){{ echo -e "${{RED}}  ✗${{NC}} $1"; }}

echo ""
echo -e "${{CYAN}}  {0} v{1}${{NC}}"
echo -e "  {2}"
echo ""

PM=""
detect_pm() {{
    for pm in pacman yay paru apt dnf zypper apk; do
        if command -v "$pm" &>/dev/null; then PM="$pm"; return 0; fi
    done
    return 1
}}

os_check() {{
    CURRENT_OS="{3}"
    LOCKED="{4}"
    if [ -n "$LOCKED" ] && [ "$CURRENT_OS" != "$LOCKED" ]; then
        err "This installer is locked to $LOCKED. Current OS: $CURRENT_OS"
        exit 1
    fi
}}

install_pkgs() {{
    local pkgs="{5}"
    case "$PM" in
        pacman|yay|paru) sudo "$PM" -S --noconfirm $pkgs ;;
        apt) sudo apt install -y $pkgs ;;
        dnf) sudo dnf install -y $pkgs ;;
        zypper) sudo zypper install -y $pkgs ;;
        apk) sudo apk add $pkgs ;;
        *) err "No supported package manager found" && exit 1 ;;
    esac
}}

main() {{
    os_check
    info "Detecting package manager..."
    detect_pm || {{ err "No supported package manager."; exit 1; }}
    success "Found: $PM"
    info "Installing packages..."
    install_pkgs
    success "{0} installed successfully!"
{6}
    echo ""
}}
main "$@"
"##,
        config.name,
        config.version,
        config.description,
        detect_os(),
        config.lock_os.as_deref().unwrap_or(""),
        pkg_list,
        config.post_install.iter().map(|c| format!("    success \"Running: {}\"\n    {}", c, c)).collect::<Vec<_>>().join("\n"),
    )
}

fn detect_os() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(id) = line.strip_prefix("ID=") {
                return id.trim_matches('"').to_lowercase();
            }
        }
    }
    std::env::consts::OS.to_string()
}

fn base64_encode(s: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn whoami() -> Option<String> {
    std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok())
}
