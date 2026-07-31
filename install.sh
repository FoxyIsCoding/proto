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

    case "${prev}" in
        proto)
            COMPREPLY=( $(compgen -W "help system pkg git setup mc" -- "${cur}") )
            return 0 ;;
        pkg)
            COMPREPLY=( $(compgen -W "install search remove update list" -- "${cur}") )
            return 0 ;;
        git)
            COMPREPLY=( $(compgen -W "log stats save undo branch" -- "${cur}") )
            return 0 ;;
        mc)
            COMPREPLY=( $(compgen -W "resource_pack server" -- "${cur}") )
            return 0 ;;
        resource_pack)
            COMPREPLY=( $(compgen -W "create fetch pack add" -- "${cur}") )
            return 0 ;;
        server)
            [[ "${word2}" == "mc" ]] && COMPREPLY=( $(compgen -W "create ping status" -- "${cur}") )
            return 0 ;;
        help)
            COMPREPLY=( $(compgen -W "help system pkg git setup mc" -- "${cur}") )
            return 0 ;;
    esac

    if [[ "${word2}" == "mc" && "${word3}" == "resource_pack" ]]; then
        COMPREPLY=( $(compgen -W "create fetch pack add" -- "${cur}") )
    elif [[ "${word2}" == "mc" && "${word3}" == "server" ]]; then
        COMPREPLY=( $(compgen -W "create ping status" -- "${cur}") )
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
        'pkg:Cross-distro package manager wrapper'
        'git:Git workflow enhancements'
        'setup:Interactive configuration wizard'
        'mc:Minecraft utilities'
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
            esac
            case $words[2] in
                resource_pack) _describe -t actions 'resource_pack' rp_actions ;;
                server)        _describe -t actions 'server' server_actions ;;
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
complete -c proto -n "__fish_use_subcommand" -a help -d "Show help"
complete -c proto -n "__fish_use_subcommand" -a system -d "System information"
complete -c proto -n "__fish_use_subcommand" -a pkg -d "Package manager wrapper"
complete -c proto -n "__fish_use_subcommand" -a git -d "Git enhancements"
complete -c proto -n "__fish_use_subcommand" -a setup -d "Configuration wizard"
complete -c proto -n "__fish_use_subcommand" -a mc -d "Minecraft utilities"

complete -c proto -n "__fish_seen_subcommand_from pkg" -a install -d "Install packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a search -d "Search packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a remove -d "Remove packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a update -d "Update packages"
complete -c proto -n "__fish_seen_subcommand_from pkg" -a list -d "List packages"

complete -c proto -n "__fish_seen_subcommand_from git" -a log -d "Pretty git log"
complete -c proto -n "__fish_seen_subcommand_from git" -a stats -d "Repo statistics"
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
