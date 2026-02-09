#!/bin/sh
# Hlavi CLI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/mmuhlariholdings/hlavi/main/hlavi-cli/install.sh | sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    printf "${BLUE}ℹ${NC} %s\n" "$1"
}

print_success() {
    printf "${GREEN}✓${NC} %s\n" "$1"
}

print_error() {
    printf "${RED}✗${NC} %s\n" "$1" >&2
}

print_warning() {
    printf "${YELLOW}⚠${NC} %s\n" "$1"
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            OS="linux"
            ;;
        Darwin)
            OS="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            ;;
        *)
            print_error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            print_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    print_info "Detected platform: $OS-$ARCH"
}

# Get latest version from GitHub
get_latest_version() {
    print_info "Fetching latest version..."

    if command -v curl > /dev/null 2>&1; then
        LATEST_VERSION=$(curl -fsSL https://api.github.com/repos/mmuhlariholdings/hlavi-cli/releases/latest | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
    elif command -v wget > /dev/null 2>&1; then
        LATEST_VERSION=$(wget -qO- https://api.github.com/repos/mmuhlariholdings/hlavi-cli/releases/latest | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    if [ -z "$LATEST_VERSION" ]; then
        print_error "Failed to fetch latest version"
        exit 1
    fi

    print_success "Latest version: v$LATEST_VERSION"
}

# Download and install
install_hlavi() {
    INSTALL_DIR="${INSTALL_DIR:-$HOME/.hlavi}"
    BIN_DIR="$INSTALL_DIR/bin"

    print_info "Installing hlavi to $BIN_DIR"

    # Create installation directory
    mkdir -p "$BIN_DIR"

    # Construct download URL
    if [ "$OS" = "windows" ]; then
        ARCHIVE="hlavi-${LATEST_VERSION}-${OS}-${ARCH}.zip"
        BINARY="hlavi.exe"
    else
        ARCHIVE="hlavi-${LATEST_VERSION}-${OS}-${ARCH}.tar.gz"
        BINARY="hlavi"
    fi

    DOWNLOAD_URL="https://github.com/mmuhlariholdings/hlavi-cli/releases/download/v${LATEST_VERSION}/${ARCHIVE}"

    print_info "Downloading from $DOWNLOAD_URL"

    # Download archive
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "/tmp/$ARCHIVE"
    else
        wget -q "$DOWNLOAD_URL" -O "/tmp/$ARCHIVE"
    fi

    # Extract archive
    print_info "Extracting..."
    if [ "$OS" = "windows" ]; then
        unzip -q "/tmp/$ARCHIVE" -d "/tmp/hlavi-install"
    else
        tar xzf "/tmp/$ARCHIVE" -C "/tmp"
    fi

    # Move binary to install location
    if [ "$OS" = "windows" ]; then
        mv "/tmp/hlavi-install/$BINARY" "$BIN_DIR/$BINARY"
    else
        mv "/tmp/$BINARY" "$BIN_DIR/$BINARY"
        chmod +x "$BIN_DIR/$BINARY"
    fi

    # Cleanup
    rm -rf "/tmp/$ARCHIVE" "/tmp/hlavi-install"

    print_success "Hlavi installed successfully!"
}

# Update PATH instructions
print_path_instructions() {
    SHELL_CONFIG=""

    case "$SHELL" in
        */bash)
            if [ -f "$HOME/.bashrc" ]; then
                SHELL_CONFIG="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                SHELL_CONFIG="$HOME/.bash_profile"
            fi
            ;;
        */zsh)
            SHELL_CONFIG="$HOME/.zshrc"
            ;;
        */fish)
            SHELL_CONFIG="$HOME/.config/fish/config.fish"
            ;;
    esac

    echo ""
    print_info "To use hlavi, add it to your PATH:"
    echo ""
    echo "  export PATH=\"\$HOME/.hlavi/bin:\$PATH\""
    echo ""

    if [ -n "$SHELL_CONFIG" ]; then
        print_info "Add this line to $SHELL_CONFIG to make it permanent."
        echo ""
        read -p "$(printf "${BLUE}?${NC} Add to $SHELL_CONFIG now? (y/N) ")" -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "" >> "$SHELL_CONFIG"
            echo "# Hlavi CLI" >> "$SHELL_CONFIG"
            echo "export PATH=\"\$HOME/.hlavi/bin:\$PATH\"" >> "$SHELL_CONFIG"
            print_success "Updated $SHELL_CONFIG"
            print_warning "Run 'source $SHELL_CONFIG' or restart your terminal to use hlavi"
        fi
    fi

    echo ""
    print_info "Verify installation:"
    echo "  hlavi --version"
    echo ""
    print_info "Get started:"
    echo "  hlavi init"
    echo "  hlavi tickets create \"Your first task\""
    echo ""
    print_info "Documentation: https://github.com/mmuhlariholdings/hlavi"
}

# Main installation flow
main() {
    echo ""
    print_info "Hlavi CLI Installer"
    echo ""

    detect_platform
    get_latest_version
    install_hlavi
    print_path_instructions

    echo ""
    print_success "Installation complete! 🚀"
    echo ""
}

main
