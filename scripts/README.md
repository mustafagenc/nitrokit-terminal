# 📦 Installation

## Quick Install

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/mustafagenc/nitrokit-terminal/main/scripts/install.ps1 | iex
```

### macOS/Linux (Bash)

```bash
curl -sSL https://raw.githubusercontent.com/mustafagenc/nitrokit-terminal/main/scripts/install.sh | bash
```

## Manual Installation

### Prerequisites

- [Rust](https://rustup.rs/) (will be installed automatically if missing)
- [Git](https://git-scm.com/) (optional, for some features)

### From Source

```bash
git clone https://github.com/mustafagenc/nitrokit-terminal.git
cd nitrokit-terminal/nitrokit
cargo build --release
cargo install --path .
```

## Usage

### Command Line

```bash
# Interactive mode
nitrokit -i

# Generate release notes
nitrokit release-notes

# Update dependencies
nitrokit update-dependencies --check-only
```

### Quick Aliases (after installation)

```bash
nk      # shortcut for nitrokit
nki     # shortcut for nitrokit -i (interactive)
```

## Uninstall

### Windows

```powershell
# Download and run uninstaller
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/mustafagenc/nitrokit-terminal/main/scripts/uninstall.ps1" -OutFile "uninstall.ps1"
.\uninstall.ps1
```

### macOS/Linux

```bash
curl -sSL https://raw.githubusercontent.com/mustafagenc/nitrokit-terminal/main/scripts/uninstall.sh | bash
```
