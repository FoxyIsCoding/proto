use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum SetupAction {}

pub fn run() {
    use dialoguer::{Confirm, Select, Input};
    use crate::utils::{self, detect_package_managers};

    println!("{}", style::proto_banner());
    println!("{}\n", "Setup Wizard".style(style::Theme::HEADER));
    println!("{}", "Let's configure Proto for your system!\n".style(style::Theme::MUTED));

    let mut config = utils::load_config();

    let color_enabled = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Enable colored output?")
        .default(true)
        .interact()
        .unwrap_or(true);

    config.color = Some(color_enabled);

    let managers = detect_package_managers();
    if managers.len() > 1 {
        let items: Vec<String> = managers.iter().map(|pm| pm.name().to_string()).collect();
        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select your preferred package manager")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(0);

        config.default_pm = Some(items[selection].clone());
    }

    let install_shell = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Install shell completions? (bash, zsh, fish)")
        .default(true)
        .interact()
        .unwrap_or(true);

    if install_shell {
        println!(); // spacing
        let shell = get_shell_name();
        match install_completions(&shell) {
            Ok(path) => {
                println!("{} Installed completions for {} at {}", style::success(""), shell, path);
                config.completions_installed = Some(true);
            }
            Err(e) => {
                println!("{} {}", style::warn(""), e);
            }
        }
    }

    let custom_dir: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Install directory (leave empty for default ~/.local/bin)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    if !custom_dir.is_empty() {
        config.install_dir = Some(custom_dir);
    }

    if let Err(e) = utils::save_config(&config) {
        eprintln!("{} Failed to save config: {}", style::error(""), e);
    } else {
        println!("\n{} {}", style::success(""), "Configuration saved!".style(style::Theme::BOLD));
        println!("{} {}", style::divider(), "");
        println!("{} {}", "To get started, try:".style(style::Theme::MUTED), "proto help".style(style::Theme::ACCENT));
        println!("{} {}", "                  ".style(style::Theme::MUTED), "proto system".style(style::Theme::ACCENT));
    }
}

fn get_shell_name() -> String {
    crate::utils::get_shell()
}

fn install_completions(shell: &str) -> Result<String, String> {
    let dir = crate::utils::config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create dir: {}", e))?;

    match shell {
        "bash" => {
            let path = dir.join("proto.bash");
            let script = generate_bash_completion();
            std::fs::write(&path, script).map_err(|e| format!("Cannot write: {}", e))?;
            Ok(path.to_string_lossy().to_string())
        }
        "zsh" => {
            let path = dir.join("_proto");
            let script = generate_zsh_completion();
            std::fs::write(&path, script).map_err(|e| format!("Cannot write: {}", e))?;
            Ok(path.to_string_lossy().to_string())
        }
        "fish" => {
            let fish_dir = dirs::home_dir()
                .unwrap_or_default()
                .join(".config/fish/completions");
            std::fs::create_dir_all(&fish_dir).map_err(|e| format!("Cannot create dir: {}", e))?;
            let path = fish_dir.join("proto.fish");
            let script = generate_fish_completion();
            std::fs::write(&path, script).map_err(|e| format!("Cannot write: {}", e))?;
            Ok(path.to_string_lossy().to_string())
        }
        _ => Err(format!("Unsupported shell: {}", shell)),
    }
}

fn generate_bash_completion() -> String {
    r#"_proto_completion() {
    local cur prev word2 word3
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    word2="${COMP_WORDS[2]}"
    word3="${COMP_WORDS[3]}"

    case "${prev}" in
        proto)
            COMPREPLY=( $(compgen -W "help system pkg git setup mc" -- "${cur}") )
            return 0
            ;;
        pkg)
            COMPREPLY=( $(compgen -W "install search remove update list" -- "${cur}") )
            return 0
            ;;
        git)
            COMPREPLY=( $(compgen -W "log stats save undo branch" -- "${cur}") )
            return 0
            ;;
        mc)
            COMPREPLY=( $(compgen -W "resource_pack server" -- "${cur}") )
            return 0
            ;;
        resource_pack)
            COMPREPLY=( $(compgen -W "create fetch pack add" -- "${cur}") )
            return 0
            ;;
        server)
            [[ "${word2}" == "mc" ]] && COMPREPLY=( $(compgen -W "create ping status" -- "${cur}") )
            return 0
            ;;
        help)
            COMPREPLY=( $(compgen -W "help system pkg git setup mc" -- "${cur}") )
            return 0
            ;;
    esac

    if [[ "${word2}" == "mc" && "${word3}" == "resource_pack" ]]; then
        COMPREPLY=( $(compgen -W "create fetch pack add" -- "${cur}") )
    elif [[ "${word2}" == "mc" && "${word3}" == "server" ]]; then
        COMPREPLY=( $(compgen -W "create ping status" -- "${cur}") )
    fi
}

complete -F _proto_completion proto
"#.to_string()
}

fn generate_zsh_completion() -> String {
    r#"#compdef proto

_proto() {
    local -a commands
    commands=(
        'help:Show help for commands'
        'system:Display system information'
        'pkg:Cross-distro package manager wrapper'
        'git:Git workflow enhancements'
        'setup:Interactive configuration wizard'
        'mc:Minecraft utilities'
        'status:Network monitoring'
    )

    local -a pkg_actions
    pkg_actions=(
        'install:Install packages'
        'search:Search packages'
        'remove:Remove packages'
        'update:Update packages'
        'list:List installed packages'
    )

    local -a git_actions
    git_actions=(
        'log:Show pretty git log'
        'stats:Show repo statistics'
        'save:Quick WIP commit'
        'undo:Undo last commit'
        'branch:Show branches'
    )

    local -a mc_actions
    mc_actions=(
        'resource_pack:Resource pack utilities'
        'server:Server management'
    )

    local -a rp_actions
    rp_actions=(
        'create:Create a resource pack'
        'fetch:Fetch versions'
        'pack:Pack into zip'
        'add:Add an asset'
    )

    local -a server_actions
    server_actions=(
        'create:Create a server'
        'ping:Ping a server'
        'status:Server status'
    )

    local -a status_actions
    status_actions=(
        'ping:Ping a host'
        'monitor:Live monitor'
        'serve:Web dashboard'
        'report:Generate report'
    )

    _arguments -C \
        '--version[Print version]' \
        '--help[Print help]' \
        '1: :_describe command commands' \
        '*::arg:->args'

    case "$state" in
        args)
            case $words[1] in
                pkg)
                    _describe -t actions 'pkg action' pkg_actions
                    ;;
                git)
                    _describe -t actions 'git action' git_actions
                    ;;
                mc)
                    _describe -t actions 'mc action' mc_actions
                    ;;
                status)
                    _describe -t actions 'status action' status_actions
                    ;;
            esac
            case $words[2] in
                resource_pack)
                    _describe -t actions 'resource_pack' rp_actions
                    ;;
                server)
                    _describe -t actions 'server' server_actions
                    ;;
            esac
            ;;
    esac
}

_proto "$@"
"#.to_string()
}

fn generate_fish_completion() -> String {
    r#"complete -c proto -f

complete -c proto -n "__fish_use_subcommand" -a help -d "Show help for commands"
complete -c proto -n "__fish_use_subcommand" -a system -d "Display system information"
complete -c proto -n "__fish_use_subcommand" -a pkg -d "Cross-distro package manager wrapper"
complete -c proto -n "__fish_use_subcommand" -a git -d "Git workflow enhancements"
complete -c proto -n "__fish_use_subcommand" -a setup -d "Interactive configuration wizard"
complete -c proto -n "__fish_use_subcommand" -a mc -d "Minecraft utilities"
complete -c proto -n "__fish_use_subcommand" -a status -d "Network monitoring"

complete -c proto -n "__fish_seen_subcommand_from pkg" -a install -d "Install packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a search -d "Search packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a remove -d "Remove packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a update -d "Update packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a list -d "List installed packages"

complete -c proto -n "__fish_seen_subcommand_from git" -a log -d "Show pretty git log"
complete -c proto -n "__fish_seen_subcommand_from git" -a stats -d "Show repo statistics"
complete -c proto -n "__fish_seen_subcommand_from git" -a save -d "Quick WIP commit"
complete -c proto -n "__fish_seen_subcommand_from git" -a undo -d "Undo last commit"
complete -c proto -n "__fish_seen_subcommand_from git" -a branch -d "Show branches"

complete -c proto -n "__fish_seen_subcommand_from mc" -a resource_pack -d "Resource pack utilities"
complete -c proto -n "__fish_seen_subcommand_from mc" -a server -d "Server management"

complete -c proto -n "__fish_seen_subcommand_from mc server" -a create -d "Create a server"
complete -c proto -n "__fish_seen_subcommand_from mc server" -a ping -d "Ping a server"
complete -c proto -n "__fish_seen_subcommand_from mc server" -a status -d "Server status"

complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a create -d "Create a resource pack"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a fetch -d "Fetch versions"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a pack -d "Pack into zip"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a add -d "Add an asset"

complete -c proto -n "__fish_seen_subcommand_from status" -a ping -d "Ping a host"
complete -c proto -n "__fish_seen_subcommand_from status" -a monitor -d "Live monitor"
complete -c proto -n "__fish_seen_subcommand_from status" -a serve -d "Web dashboard"
complete -c proto -n "__fish_seen_subcommand_from status" -a report -d "Generate report"

complete -c proto -l version -d "Print version"
complete -c proto -l help -d "Print help"
"#.to_string()
}
