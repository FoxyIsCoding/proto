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

PROTO_VERSION="0.1.0"
BIN_NAME="proto"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"
SYSTEM_INSTALL_DIR="/usr/local/bin"

echo ""
echo -e "${CYAN}      /‾‾‾‾‾‾/${NC}"
echo -e "${CYAN}     /  ◈ ◈  /${NC}"
echo -e "${CYAN}    /  ▔▔▔▔  /${NC}"
echo -e "${CYAN}   /________/${NC}"
echo -e "${CYAN}   | □  □  |${NC}"
echo -e "${CYAN}   |   ▼   |${NC}"
echo -e "${CYAN}   |_______|${NC}"
echo ""
echo -e "${BOLD}${CYAN}  Proto CLI ${WHITE}v${PROTO_VERSION}${NC}"
echo -e "  ${BLUE}Your friendly protogen CLI companion${NC}"
echo ""
echo -e "  ${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

info()  { echo -e "${CYAN}  ◆${NC} $1"; }
success() { echo -e "${GREEN}  ✔${NC} $1"; }
warn()  { echo -e "${YELLOW}  ⚠${NC}  $1"; }
err()   { echo -e "${RED}  ✗${NC} $1"; }

check_rust() {
    if command -v cargo &>/dev/null; then
        success "Rust is installed ($(rustc --version | cut -d' ' -f2))"
        return 0
    fi

    warn "Rust is not installed."
    echo ""
    read -r -p "  Install Rust via rustup? [Y/n] " answer
    answer="${answer:-Y}"

    if [[ "$answer" =~ ^[Yy]$ ]]; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        success "Rust installed successfully!"
    else
        err "Rust is required to build Proto CLI."
        exit 1
    fi
}

build_proto() {
    info "Building Proto CLI (this may take a moment)..."
    cargo build --release 2>&1 | while IFS= read -r line; do
        if [[ "$line" =~ "Compiling" || "$line" =~ "Building" ]]; then
            echo -e "  ${BLUE}▸${NC} ${line}" >&2
        fi
    done

    if [[ ! -f "target/release/$BIN_NAME" ]]; then
        err "Build failed. Check the output above for errors."
        exit 1
    fi
    success "Build complete!"
}

install_binary() {
    echo ""
    info "Where should Proto be installed?"
    echo "  1) ${DEFAULT_INSTALL_DIR} (user only, no sudo required)"
    echo "  2) ${SYSTEM_INSTALL_DIR} (system-wide, requires sudo)"
    echo ""
    read -r -p "  Choice [1-2] (default: 1): " choice
    choice="${choice:-1}"

    case "$choice" in
        1) INSTALL_DIR="$DEFAULT_INSTALL_DIR" ;;
        2) INSTALL_DIR="$SYSTEM_INSTALL_DIR" ;;
        *) INSTALL_DIR="$DEFAULT_INSTALL_DIR" ;;
    esac

    mkdir -p "$INSTALL_DIR"

    if [[ "$INSTALL_DIR" == "/usr/local/bin" ]]; then
        info "Installing system-wide (sudo required)..."
        sudo cp "target/release/$BIN_NAME" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/$BIN_NAME"
    else
        info "Installing to $INSTALL_DIR..."
        cp "target/release/$BIN_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BIN_NAME"
    fi

    success "Proto installed to $INSTALL_DIR/$BIN_NAME"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        warn "$INSTALL_DIR is not in your PATH."
        echo ""
        echo "  Add this to your shell config (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo -e "    ${CYAN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo ""
    fi
}

setup_completions() {
    echo ""
    read -r -p "  Install shell completions? [Y/n] " answer
    answer="${answer:-Y}"

    if [[ ! "$answer" =~ ^[Yy]$ ]]; then
        return
    fi

    local shell
    shell="$(basename "$SHELL")"

    local completions_dir="$HOME/.local/share/proto/completions"
    mkdir -p "$completions_dir"

    info "Generating completions for $shell..."

    "$INSTALL_DIR/$BIN_NAME" completions "$shell" > "$completions_dir/proto.$shell" 2>/dev/null || {
        warn "Auto-generation failed. You can manually set up completions later."
        return
    }

    success "Completions installed to $completions_dir"

    case "$shell" in
        bash)
            echo "  Add to ~/.bashrc:"
            echo -e "    ${CYAN}[ -f $completions_dir/proto.bash ] && source $completions_dir/proto.bash${NC}"
            ;;
        zsh)
            echo "  Add to ~/.zshrc:"
            echo -e "    ${CYAN}fpath=($completions_dir \$fpath)${NC}"
            ;;
        fish)
            echo "  Symlink to completions:"
            echo -e "    ${CYAN}ln -s $completions_dir/proto.fish ~/.config/fish/completions/${NC}"
            ;;
    esac
    echo ""
}

run_setup() {
    echo ""
    read -r -p "  Run interactive setup wizard? [Y/n] " answer
    answer="${answer:-Y}"

    if [[ "$answer" =~ ^[Yy]$ ]]; then
        "$INSTALL_DIR/$BIN_NAME" setup
    fi
}

main() {
    echo ""

    check_rust

    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    cd "$SCRIPT_DIR"

    build_proto
    install_binary
    setup_completions
    run_setup

    echo ""
    echo -e "  ${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  ${GREEN}✦ Proto CLI installed successfully! ✦${NC}"
    echo ""
    echo "  Get started:"
    echo -e "    ${CYAN}proto help${NC}       Show all commands"
    echo -e "    ${CYAN}proto system${NC}     View system information"
    echo -e "    ${CYAN}proto pkg install${NC} Install packages"
    echo -e "    ${CYAN}proto git log${NC}     Pretty git history"
    echo ""
    echo -e "  ${BLUE}Enjoy! 🦊${NC}"
    echo ""
}

main "$@"
