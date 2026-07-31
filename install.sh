#!/usr/bin/env bash
set -euo pipefail

BOLD="\033[1m"
CYAN="\033[0;36m"
BLUE="\033[0;34m"
WHITE="\033[0;37m"
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
RED="\033[0;31m"
NC="\033[0m"
DIV="──────────────────────────────────────────"

PROTO_VERSION="0.1.0"
BIN_NAME="proto"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
SYSTEM_INSTALL_DIR="/usr/local/bin"

info()    { echo -e "${CYAN}  ◆${NC} $1"; }
success() { echo -e "${GREEN}  ✔${NC} $1"; }
warn()    { echo -e "${YELLOW}  ⚠${NC}  $1"; }
err()     { echo -e "${RED}  ✗${NC} $1"; }
sep()     { echo -e "\n  ${BLUE}${DIV}${NC}\n"; }

check_rust() {
    if command -v cargo &>/dev/null; then
        success "Rust is installed ($(rustc --version | cut -d' ' -f2))"
        return 0
    fi

    warn "Rust is not installed."
    read -r -p "  Install Rust via rustup? [Y/n] " answer
    answer="${answer:-Y}"
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        success "Rust installed"
    else
        err "Rust is required to build Proto CLI."
        exit 1
    fi
}

build_proto() {
    info "Building Proto CLI..."
    cargo build --release 2>&1 | while IFS= read -r line; do
        if [[ "$line" =~ "Compiling" || "$line" =~ "Building" ]]; then
            echo -e "  ${BLUE}▸${NC} ${line}" >&2
        fi
    done
    if [[ ! -f "target/release/$BIN_NAME" ]]; then
        err "Build failed."
        exit 1
    fi
    success "Build complete"
}

install_binary() {
    info "Where should Proto be installed?"
    echo "  1) ${DEFAULT_INSTALL_DIR}  (user only, no sudo)"
    echo "  2) ${SYSTEM_INSTALL_DIR}   (system-wide, needs sudo)"
    read -r -p "  Choice [1-2] (default: 1): " choice
    choice="${choice:-1}"

    case "$choice" in
        1) INSTALL_DIR="$DEFAULT_INSTALL_DIR" ;;
        2) INSTALL_DIR="$SYSTEM_INSTALL_DIR" ;;
        *) INSTALL_DIR="$DEFAULT_INSTALL_DIR" ;;
    esac

    mkdir -p "$INSTALL_DIR"

    if [[ "$INSTALL_DIR" == "/usr/local/bin" ]]; then
        info "Installing system-wide..."
        sudo cp "target/release/$BIN_NAME" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/$BIN_NAME"
    else
        info "Installing to $INSTALL_DIR..."
        cp "target/release/$BIN_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BIN_NAME"
    fi
    success "Installed: $INSTALL_DIR/$BIN_NAME"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        warn "$INSTALL_DIR is not in your PATH."
        echo "  Add to your shell config:"
        echo -e "    ${CYAN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
    fi
}

setup_completions() {
    read -r -p "  Install shell completions? [Y/n] " answer
    answer="${answer:-Y}"
    [[ "$answer" =~ ^[Yy]$ ]] || return

    local shell
    shell="$(basename "$SHELL")"

    info "Generating completions for $shell..."

    case "$shell" in
        bash)
            write_bash_completion
            echo "  Add to ~/.bashrc:"
            echo -e "    ${CYAN}source ~/.local/share/proto/completions/proto.bash${NC}"
            ;;
        zsh)
            write_zsh_completion
            echo "  Add to ~/.zshrc:"
            echo -e "    ${CYAN}fpath=(~/.local/share/proto/completions \$fpath)${NC}"
            ;;
        fish)
            write_fish_completion
            echo "  Restart your shell for completions to take effect."
            ;;                                                                                                                                                                                  
        *)
            warn "Unsupported shell: $shell"
            return
            ;;
    esac
    success "Completions installed"
}

write_bash_completion() {
    local dir="$HOME/.local/share/proto/completions"
    mkdir -p "$dir"
    cat > "$dir/proto.bash" << 'BASH_EOF'
_proto_completion() {
    local cur prev word2 word3
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    word2="${COMP_WORDS[2]}"
    word3="${COMP_WORDS[3]}"

    local commands="help system alias share-session pkg git setup mc status discord convert encrypt app ai copy-ctx memo secret webhook pr-prep pr-checkout git-who-broke git-impact git-catchup share dedupe media cert-check dns-lookup local-s3 port-forward"
    local pkg_actions="install search remove update list build"
    local pack_actions="create edit build test"
    local git_actions="log stats save undo branch"
    local mc_actions="resource_pack server"
    local rp_actions="create fetch pack add"
    local server_actions="create ping status"
    local status_actions="ping monitor serve report"
    local encrypt_actions="base64 hex hash uuid bcrypt"
    local app_actions="doctor port nuke snap"
    local ai_actions="setup chat summarize explain"
    local memo_actions="add list clear"
    local alias_actions="create list remove"
    local media_actions="shrink"

    case "${prev}" in
        proto) COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") ); return 0 ;;
        pkg) COMPREPLY=( $(compgen -W "${pkg_actions}" -- "${cur}") ); return 0 ;;
        build) COMPREPLY=( $(compgen -W "pack" -- "${cur}") ); return 0 ;;
        pack) COMPREPLY=( $(compgen -W "${pack_actions}" -- "${cur}") ); return 0 ;;
        git) COMPREPLY=( $(compgen -W "${git_actions}" -- "${cur}") ); return 0 ;;
        mc) COMPREPLY=( $(compgen -W "${mc_actions}" -- "${cur}") ); return 0 ;;
        resource_pack) COMPREPLY=( $(compgen -W "${rp_actions}" -- "${cur}") ); return 0 ;;
        server) [[ "${word2}" == "mc" ]] && COMPREPLY=( $(compgen -W "${server_actions}" -- "${cur}") ); return 0 ;;
        status) COMPREPLY=( $(compgen -W "${status_actions}" -- "${cur}") ); return 0 ;;
        discord) COMPREPLY=( $(compgen -W "bot quest" -- "${cur}") ); return 0 ;;
        bot) COMPREPLY=( $(compgen -W "create" -- "${cur}") ); return 0 ;;
        encrypt) COMPREPLY=( $(compgen -W "${encrypt_actions}" -- "${cur}") ); return 0 ;;
        app) COMPREPLY=( $(compgen -W "${app_actions}" -- "${cur}") ); return 0 ;;
        port) COMPREPLY=( $(compgen -W "release" -- "${cur}") ); return 0 ;;
        snap) COMPREPLY=( $(compgen -W "create restore view delete" -- "${cur}") ); return 0 ;;
        ai) COMPREPLY=( $(compgen -W "${ai_actions}" -- "${cur}") ); return 0 ;;
        memo) COMPREPLY=( $(compgen -W "${memo_actions}" -- "${cur}") ); return 0 ;;
        alias) COMPREPLY=( $(compgen -W "${alias_actions}" -- "${cur}") ); return 0 ;;
        share-session) COMPREPLY=( $(compgen -W "create join" -- "${cur}") ); return 0 ;;
        secret) COMPREPLY=( $(compgen -W "mask" -- "${cur}") ); return 0 ;;
        webhook) COMPREPLY=( $(compgen -W "listen" -- "${cur}") ); return 0 ;;
        media) COMPREPLY=( $(compgen -W "${media_actions}" -- "${cur}") ); return 0 ;;
        help) COMPREPLY=( $(compgen -W "${commands}" -- "${cur}") ); return 0 ;;
    esac

    if [[ "${word2}" == "mc" && "${word3}" == "resource_pack" ]]; then
        COMPREPLY=( $(compgen -W "${rp_actions}" -- "${cur}") )
    elif [[ "${word2}" == "mc" && "${word3}" == "server" ]]; then
        COMPREPLY=( $(compgen -W "${server_actions}" -- "${cur}") )
    elif [[ "${word2}" == "pkg" && "${word3}" == "build" ]]; then
        COMPREPLY=( $(compgen -W "pack" -- "${cur}") )
    elif [[ "${word2}" == "pkg" && "${word3}" == "pack" ]]; then
        COMPREPLY=( $(compgen -W "${pack_actions}" -- "${cur}") )
    fi
}
complete -F _proto_completion proto
BASH_EOF
}

write_zsh_completion() {
    local dir="$HOME/.local/share/proto/completions"
    mkdir -p "$dir"
    cat > "$dir/_proto" << 'ZSH_EOF'
#compdef proto

_proto() {
    local -a commands
    commands=(
        'help:Show help for commands'
        'system:Display system information'
        'alias:Interactive shell alias builder'
        'share-session:Share terminal session'
        'pkg:Package manager wrapper'
        'git:Git workflow enhancements'
        'setup:Interactive configuration wizard'
        'mc:Minecraft utilities'
        'status:Network monitoring'
        'discord:Discord bot creator'
        'convert:Convert units'
        'encrypt:Encode/hash/generate crypto'
        'app:Project diagnostics & cleanup'
        'ai:AI assistant'
        'copy-ctx:Bundle repo into clipboard context'
        'memo:Location-aware memos'
        'secret:Scan for leaked secrets'
        'webhook:Listen for webhooks'
        'pr-prep:PR readiness check'
        'pr-checkout:Check out a PR locally'
        'git-who-broke:Bisect to find breaking commit'
        'git-impact:Branch risk score'
        'git-catchup:Upstream changes since last pull'
        'share:Upload file to a temporary host'
        'dedupe:Find exact duplicate files'
        'media:Compress images/videos in place'
        'cert-check:Inspect a remote TLS certificate'
        'dns-lookup:Query DNS records'
        'local-s3:Spin up an ephemeral MinIO server'
        'port-forward:SSH port forwarding with auto-retry'
    )
    local -a pkg_actions
    pkg_actions=(
        'install:Install packages'
        'search:Search packages'
        'remove:Remove packages'
        'update:Update packages'
        'list:List installed packages'
        'build:Build pack tools'
    )
    local -a pack_actions
    pack_actions=(
        'create:Create pack config'
        'edit:Edit pack config'
        'build:Generate installer'
        'test:Dry-run pack config'
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
    local -a encrypt_actions
    encrypt_actions=(
        'base64:Base64 encode/decode'
        'hex:Hex encode/decode'
        'hash:Hash text'
        'uuid:Generate UUID v4'
        'bcrypt:Bcrypt hash a password'
    )
    local -a app_actions
    app_actions=(
        'doctor:Audit project health'
        'port:Port management'
        'nuke:Purge build artifacts'
        'snap:Git snapshots'
    )
    local -a ai_actions
    ai_actions=(
        'setup:Configure AI provider'
        'chat:Interactive chat'
        'summarize:Changelog from git log'
        'explain:Explain last failed command'
    )
    local -a memo_actions
    memo_actions=(
        'add:Add a memo'
        'list:Show memos'
        'clear:Clear memos'
    )
    local -a alias_actions
    alias_actions=(
        'create:Create an alias'
        'list:List aliases'
        'remove:Remove an alias'
    )
    local -a media_actions
    media_actions=(
        'shrink:Compress media files'
    )
    _arguments -C \
        '--version[Print version]' \
        '--help[Print help]' \
        '1: :_describe command commands' \
        '*::arg:->args'
    case "$state" in
        args)
            case $words[1] in
                pkg) _describe -t actions 'pkg action' pkg_actions ;;
                git) _describe -t actions 'git action' git_actions ;;
                mc)  _describe -t actions 'mc action' mc_actions ;;
                status) _describe -t actions 'status action' status_actions ;;
                encrypt) _describe -t actions 'encrypt action' encrypt_actions ;;
                app) _describe -t actions 'app action' app_actions ;;
                ai) _describe -t actions 'ai action' ai_actions ;;
                memo) _describe -t actions 'memo action' memo_actions ;;
                alias) _describe -t actions 'alias action' alias_actions ;;
                media) _describe -t actions 'media action' media_actions ;;
            esac
            case $words[2] in
                resource_pack) _describe -t actions 'resource_pack' rp_actions ;;
                server)        _describe -t actions 'server' server_actions ;;
                pack)          _describe -t actions 'pack' pack_actions ;;
            esac
            ;;
    esac
}
_proto "$@"
ZSH_EOF
}

write_fish_completion() {
    local dir="$HOME/.config/fish/completions"
    mkdir -p "$dir"
    cat > "$dir/$BIN_NAME.fish" << 'FISH_EOF'
complete -c proto -f
complete -c proto -n "__fish_use_subcommand" -a help -d "Show help for commands"
complete -c proto -n "__fish_use_subcommand" -a system -d "Display system information"
complete -c proto -n "__fish_use_subcommand" -a alias -d "Interactive shell alias builder"
complete -c proto -n "__fish_use_subcommand" -a share-session -d "Share terminal session"
complete -c proto -n "__fish_use_subcommand" -a pkg -d "Package manager wrapper"
complete -c proto -n "__fish_use_subcommand" -a git -d "Git workflow enhancements"
complete -c proto -n "__fish_use_subcommand" -a setup -d "Configuration wizard"
complete -c proto -n "__fish_use_subcommand" -a mc -d "Minecraft utilities"
complete -c proto -n "__fish_use_subcommand" -a status -d "Network monitoring"
complete -c proto -n "__fish_use_subcommand" -a discord -d "Discord bot creator"
complete -c proto -n "__fish_use_subcommand" -a convert -d "Convert units"
complete -c proto -n "__fish_use_subcommand" -a encrypt -d "Encode/hash/generate crypto"
complete -c proto -n "__fish_use_subcommand" -a app -d "Project diagnostics & cleanup"
complete -c proto -n "__fish_use_subcommand" -a ai -d "AI assistant"
complete -c proto -n "__fish_use_subcommand" -a copy-ctx -d "Bundle repo into clipboard context"
complete -c proto -n "__fish_use_subcommand" -a memo -d "Location-aware memos"
complete -c proto -n "__fish_use_subcommand" -a secret -d "Scan for leaked secrets"
complete -c proto -n "__fish_use_subcommand" -a webhook -d "Listen for webhooks"
complete -c proto -n "__fish_use_subcommand" -a pr-prep -d "PR readiness check"
complete -c proto -n "__fish_use_subcommand" -a pr-checkout -d "Check out a PR locally"
complete -c proto -n "__fish_use_subcommand" -a git-who-broke -d "Bisect to find breaking commit"
complete -c proto -n "__fish_use_subcommand" -a git-impact -d "Branch risk score"
complete -c proto -n "__fish_use_subcommand" -a git-catchup -d "Upstream changes since last pull"
complete -c proto -n "__fish_use_subcommand" -a share -d "Upload file to a temporary host"
complete -c proto -n "__fish_use_subcommand" -a dedupe -d "Find exact duplicate files"
complete -c proto -n "__fish_use_subcommand" -a media -d "Compress images/videos in place"
complete -c proto -n "__fish_use_subcommand" -a cert-check -d "Inspect a remote TLS certificate"
complete -c proto -n "__fish_use_subcommand" -a dns-lookup -d "Query DNS records"
complete -c proto -n "__fish_use_subcommand" -a local-s3 -d "Spin up an ephemeral MinIO server"
complete -c proto -n "__fish_use_subcommand" -a port-forward -d "SSH port forwarding with auto-retry"

complete -c proto -n "__fish_seen_subcommand_from pkg" -a install -d "Install packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a search -d "Search packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a remove -d "Remove packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a update -d "Update packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a list -d "List installed packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a build -d "Build pack tools"
complete -c proto -n "__fish_seen_subcommand_from build" -a pack -d "Portable installer pack"
complete -c proto -n "__fish_seen_subcommand_from pack" -a create -d "Create pack config"
complete -c proto -n "__fish_seen_subcommand_from pack" -a edit -d "Edit pack config"
complete -c proto -n "__fish_seen_subcommand_from pack" -a build -d "Generate installer"
complete -c proto -n "__fish_seen_subcommand_from pack" -a test -d "Dry-run pack config"

complete -c proto -n "__fish_seen_subcommand_from git" -a log -d "Pretty git log"
complete -c proto -n "__fish_seen_subcommand_from git" -a stats -d "Repo statistics"
complete -c proto -n "__fish_seen_subcommand_from git" -a save -d "Quick WIP commit"
complete -c proto -n "__fish_seen_subcommand_from git" -a undo -d "Undo last commit"
complete -c proto -n "__fish_seen_subcommand_from git" -a branch -d "Show branches"

complete -c proto -n "__fish_seen_subcommand_from mc" -a resource_pack -d "Resource pack utilities"
complete -c proto -n "__fish_seen_subcommand_from mc" -a server -d "Server management"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a create -d "Create a resource pack"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a fetch -d "Fetch versions"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a pack -d "Pack into zip"
complete -c proto -n "__fish_seen_subcommand_from mc resource_pack" -a add -d "Add an asset"
complete -c proto -n "__fish_seen_subcommand_from mc server" -a create -d "Create a server"
complete -c proto -n "__fish_seen_subcommand_from mc server" -a ping -d "Ping a server"
complete -c proto -n "__fish_seen_subcommand_from mc server" -a status -d "Server status"

complete -c proto -n "__fish_seen_subcommand_from status" -a ping -d "Ping a host"
complete -c proto -n "__fish_seen_subcommand_from status" -a monitor -d "Live monitor"
complete -c proto -n "__fish_seen_subcommand_from status" -a serve -d "Web dashboard"
complete -c proto -n "__fish_seen_subcommand_from status" -a report -d "Generate report"

complete -c proto -n "__fish_seen_subcommand_from discord" -a bot -d "Bot project management"
complete -c proto -n "__fish_seen_subcommand_from discord" -a quest -d "Quest completion injector"
complete -c proto -n "__fish_seen_subcommand_from bot" -a create -d "Create a bot project"

complete -c proto -n "__fish_seen_subcommand_from encrypt" -a base64 -d "Base64 encode/decode"
complete -c proto -n "__fish_seen_subcommand_from encrypt" -a hex -d "Hex encode/decode"
complete -c proto -n "__fish_seen_subcommand_from encrypt" -a hash -d "Hash text"
complete -c proto -n "__fish_seen_subcommand_from encrypt" -a uuid -d "Generate UUID v4"
complete -c proto -n "__fish_seen_subcommand_from encrypt" -a bcrypt -d "Bcrypt hash a password"

complete -c proto -n "__fish_seen_subcommand_from app" -a doctor -d "Audit project health"
complete -c proto -n "__fish_seen_subcommand_from app" -a port -d "Port management"
complete -c proto -n "__fish_seen_subcommand_from app" -a nuke -d "Purge build artifacts"
complete -c proto -n "__fish_seen_subcommand_from app" -a snap -d "Git snapshots"
complete -c proto -n "__fish_seen_subcommand_from port" -a release -d "Free a port"
complete -c proto -n "__fish_seen_subcommand_from snap" -a create -d "Create snapshot"
complete -c proto -n "__fish_seen_subcommand_from snap" -a restore -d "Restore snapshot"
complete -c proto -n "__fish_seen_subcommand_from snap" -a view -d "View snapshot"
complete -c proto -n "__fish_seen_subcommand_from snap" -a delete -d "Delete snapshot"

complete -c proto -n "__fish_seen_subcommand_from ai" -a setup -d "Configure AI provider"
complete -c proto -n "__fish_seen_subcommand_from ai" -a chat -d "Interactive chat"
complete -c proto -n "__fish_seen_subcommand_from ai" -a summarize -d "Changelog from git log"
complete -c proto -n "__fish_seen_subcommand_from ai" -a explain -d "Explain last failed command"

complete -c proto -n "__fish_seen_subcommand_from memo" -a add -d "Add a memo"
complete -c proto -n "__fish_seen_subcommand_from memo" -a list -d "Show memos"
complete -c proto -n "__fish_seen_subcommand_from memo" -a clear -d "Clear memos"

complete -c proto -n "__fish_seen_subcommand_from alias" -a create -d "Create an alias"
complete -c proto -n "__fish_seen_subcommand_from alias" -a list -d "List aliases"
complete -c proto -n "__fish_seen_subcommand_from alias" -a remove -d "Remove an alias"

complete -c proto -n "__fish_seen_subcommand_from share-session" -a create -d "Create a session"
complete -c proto -n "__fish_seen_subcommand_from share-session" -a join -d "Join a session"

complete -c proto -n "__fish_seen_subcommand_from secret" -a mask -d "Scan and mask secrets"

complete -c proto -n "__fish_seen_subcommand_from webhook" -a listen -d "Listen for webhooks"

complete -c proto -n "__fish_seen_subcommand_from media" -a shrink -d "Compress media files"

complete -c proto -l version -d "Print version"
complete -c proto -l help -d "Print help"
FISH_EOF
}

run_setup() {
    read -r -p "  Run interactive setup wizard? [Y/n] " answer
    answer="${answer:-Y}"
    [[ "$answer" =~ ^[Yy]$ ]] && "$INSTALL_DIR/$BIN_NAME" setup
}

main() {
    echo ""
    echo -e "${CYAN}    ⣀⡀    ${NC}"
    echo -e "${CYAN}⢠⣤⡀⣾⣿⣿⠀⣤⣤⡄${NC}"
    echo -e "${CYAN}⢿⣿⡇⠘⠛⠁⢸⣿⣿⠃${NC}"
    echo -e "${CYAN}⠈⣉⣤⣾⣿⣿⡆⠉⣴⣶⣶${NC}"
    echo -e "${CYAN}⣾⣿⣿⣿⣿⣿⣿⡀⠻⠟⠃${NC}"
    echo -e "${CYAN}⠙⠛⠻⢿⣿⣿⣿⡇  ${NC}"
    echo -e "${CYAN}    ⠈⠙⠋⠁  ${NC}"
    echo ""
    echo -e "${BOLD}${CYAN}Proto CLI ${WHITE}v${PROTO_VERSION}${NC}"
    echo -e "${BLUE}Your friendly protogen CLI companion${NC}"

    sep
    check_rust

    sep
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    cd "$SCRIPT_DIR"
    build_proto

    sep
    install_binary

    sep
    setup_completions

    sep
    run_setup

    sep
    echo -e "  ${GREEN}✦ Proto CLI installed successfully ✦${NC}"
    echo ""
    echo "        proto help       Show all commands"
    echo "        proto system     View system information"
    echo "        proto pkg install Install packages"
    echo "        proto git log     Pretty git history"
    echo ""
}
main "$@"
