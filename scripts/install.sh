#!/bin/bash

# Nitrokit macOS/Linux Installer
# Bash script to install Nitrokit on Unix-like systems

set -e

# Configuration
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
FORCE_INSTALL="${FORCE_INSTALL:-false}"
ADD_TO_SHELL="${ADD_TO_SHELL:-true}"
BUILD_FROM_SOURCE="${BUILD_FROM_SOURCE:-true}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# Banner
print_banner() {
    echo -e "${CYAN}"
    cat << "EOF"
    ███╗   ██╗██╗████████╗██████╗  ██████╗ ██╗  ██╗██╗████████╗
    ████╗  ██║██║╚══██╔══╝██╔══██╗██╔═══██╗██║ ██╔╝██║╚══██╔══╝
    ██╔██╗ ██║██║   ██║   ██████╔╝██║   ██║█████╔╝ ██║   ██║   
    ██║╚██╗██║██║   ██║   ██╔══██╗██║   ██║██╔═██╗ ██║   ██║   
    ██║ ╚████║██║   ██║   ██║  ██║╚██████╔╝██║  ██╗██║   ██║   
    ╚═╝  ╚═══╝╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝   ╚═╝   

    🚀 Nitrokit macOS/Linux Installer
    A terminal tool for project management and automation

EOF
    echo -e "${NC}"
}

# Helper functions
log_info() {
    echo -e "${BLUE}$1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check system requirements
check_requirements() {
    log_info "🔍 Checking system requirements..."
    
    # Check for Rust
    if command -v cargo >/dev/null 2>&1; then
        RUST_VERSION=$(cargo --version)
        log_success "Rust found: $RUST_VERSION"
    else
        log_warning "Rust not found. Installing Rust..."
        install_rust
    fi
    
    # Check for Git
    if command -v git >/dev/null 2>&1; then
        GIT_VERSION=$(git --version)
        log_success "Git found: $GIT_VERSION"
    else
        log_warning "Git not found. Please install Git:"
        echo "  • macOS: brew install git or from https://git-scm.com/"
        echo "  • Ubuntu/Debian: sudo apt install git"
        echo "  • CentOS/RHEL: sudo yum install git"
        echo ""
        read -p "Continue without Git? (some features may not work) [y/N]: " -r
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Install Rust
install_rust() {
    log_info "📥 Downloading Rust installer..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the cargo environment
    source "$HOME/.cargo/env"
    
    log_success "Rust installation completed!"
}

# Create installation directory
create_install_dir() {
    log_info "📁 Creating installation directory: $INSTALL_DIR"
    
    if [[ -d "$INSTALL_DIR" ]] && [[ "$FORCE_INSTALL" != "true" ]]; then
        read -p "Installation directory already exists. Continue? [y/N]: " -r
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_error "Installation cancelled."
            exit 1
        fi
    fi
    
    mkdir -p "$INSTALL_DIR"
    log_success "Installation directory created!"
}

# Build or download Nitrokit
install_nitrokit() {
    local nitrokit_binary="$INSTALL_DIR/nitrokit"
    
    if [[ "$BUILD_FROM_SOURCE" == "true" ]]; then
        log_info "🏗️  Building Nitrokit from source..."
        
        # Create temporary directory
        local temp_dir=$(mktemp -d)
        cd "$temp_dir"
        
        log_info "📥 Cloning repository..."
        git clone https://github.com/mustafagenc/nitrokit-terminal.git
        
        cd nitrokit-terminal/nitrokit
        
        log_info "🔨 Compiling Nitrokit..."
        cargo build --release
        
        log_info "📦 Installing binary..."
        cp target/release/nitrokit "$nitrokit_binary"
        
        # Cleanup
        rm -rf "$temp_dir"
    else
        # Future: download pre-built binary
        log_info "📥 Downloading Nitrokit binary..."
        # curl -L -o "$nitrokit_binary" "https://github.com/mustafagenc/nitrokit/releases/latest/download/nitrokit-$(uname -s)-$(uname -m)"
    fi
    
    # Make executable
    chmod +x "$nitrokit_binary"
    
    # Verify installation
    if [[ -x "$nitrokit_binary" ]]; then
        log_success "Nitrokit binary installed successfully!"
    else
        log_error "Failed to install Nitrokit binary!"
        exit 1
    fi
}

# Add to shell PATH
add_to_path() {
    if [[ "$ADD_TO_SHELL" != "true" ]]; then
        return
    fi
    
    log_info "🔧 Adding Nitrokit to PATH..."
    
    # Detect shell
    local shell_name=$(basename "$SHELL")
    local shell_rc=""
    
    case "$shell_name" in
        bash)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                shell_rc="$HOME/.bash_profile"
            else
                shell_rc="$HOME/.bashrc"
            fi
            ;;
        zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            log_warning "Unknown shell: $shell_name. Please manually add $INSTALL_DIR to your PATH."
            return
            ;;
    esac
    
    # Check if already in PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        log_success "Already in PATH!"
        return
    fi
    
    # Add to shell configuration
    if [[ "$shell_name" == "fish" ]]; then
        echo "set -gx PATH $INSTALL_DIR \$PATH" >> "$shell_rc"
    else
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_rc"
    fi
    
    log_success "Added to PATH! (restart your terminal or run 'source $shell_rc')"
}

# Create alias for easy access
create_alias() {
    log_info "🔗 Creating helpful aliases..."
    
    local shell_name=$(basename "$SHELL")
    local shell_rc=""
    
    case "$shell_name" in
        bash)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                shell_rc="$HOME/.bash_profile"
            else
                shell_rc="$HOME/.bashrc"
            fi
            ;;
        zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
    esac
    
    if [[ -n "$shell_rc" ]]; then
        if [[ "$shell_name" == "fish" ]]; then
            echo "alias nk='nitrokit'" >> "$shell_rc"
            echo "alias nki='nitrokit -i'" >> "$shell_rc"
        else
            echo "alias nk='nitrokit'" >> "$shell_rc"
            echo "alias nki='nitrokit -i'" >> "$shell_rc"
        fi
        
        log_success "Aliases created! Use 'nk' or 'nki' for quick access."
    fi
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --force)
            FORCE_INSTALL="true"
            shift
            ;;
        --no-path)
            ADD_TO_SHELL="false"
            shift
            ;;
        --download-binary)
            BUILD_FROM_SOURCE="false"
            shift
            ;;
        --help)
            echo "Nitrokit Installer"
            echo ""
            echo "Options:"
            echo "  --install-dir DIR     Installation directory (default: \$HOME/.local/bin)"
            echo "  --force              Force installation"
            echo "  --no-path            Don't add to PATH"
            echo "  --download-binary    Download pre-built binary instead of building"
            echo "  --help               Show this help"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Main installation process
main() {
    print_banner
    
    log_info "Starting Nitrokit installation..."
    log_info "Installation directory: $INSTALL_DIR"
    echo ""
    
    check_requirements
    create_install_dir
    install_nitrokit
    add_to_path
    create_alias
    
    echo ""
    echo -e "${GREEN}🎉 Installation completed successfully!${NC}"
    echo ""
    echo -e "${CYAN}📍 Installation location:${NC} $INSTALL_DIR"
    echo -e "${CYAN}🚀 Usage:${NC}"
    echo "   • Command line: nitrokit"
    echo "   • Interactive mode: nitrokit -i"
    echo "   • Quick aliases: nk, nki"
    echo "   • Generate release notes: nitrokit release-notes"
    echo "   • Update dependencies: nitrokit update-dependencies"
    echo ""
    echo -e "${CYAN}📚 Documentation:${NC} https://github.com/mustafagenc/nitrokit-terminal"
    echo -e "${CYAN}🐛 Issues:${NC} https://github.com/mustafagenc/nitrokit-/issues"
    echo ""
    
    if [[ "$ADD_TO_SHELL" == "true" ]]; then
        echo -e "${YELLOW}💡 Don't forget to restart your terminal or run 'source ~/.bashrc' (or equivalent)${NC}"
        echo ""
    fi
}

# Run main function
main "$@"