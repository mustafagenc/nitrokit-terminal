# 📦 Installation

## Quick Install

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/mustafagenc/nitroterm-terminal/main/scripts/install.ps1 | iex
```

### macOS/Linux (Bash)

```bash
curl -sSL https://raw.githubusercontent.com/mustafagenc/nitroterm-terminal/main/scripts/install.sh | bash
```

## Manual Installation

### Prerequisites

- [Rust](https://rustup.rs/) (will be installed automatically if missing)
- [Git](https://git-scm.com/) (optional, for some features)

### From Source

```bash
git clone https://github.com/mustafagenc/nitroterm-terminal.git
cd nitroterm-terminal/nitroterm
cargo build --release
cargo install --path .
```

## Usage

### Command Line

```bash
# Interactive mode
nitroterm -i

# Generate release notes
nitroterm release-notes

# Update dependencies
nitroterm update-dependencies --check-only
```

### Quick Aliases (after installation)

```bash
nk      # shortcut for nitroterm
nki     # shortcut for nitroterm -i (interactive)
```

## Uninstall

### Windows

```powershell
# Download and run uninstaller
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/mustafagenc/nitroterm-terminal/main/scripts/uninstall.ps1" -OutFile "uninstall.ps1"
.\uninstall.ps1
```

### macOS/Linux

```bash
curl -sSL https://raw.githubusercontent.com/mustafagenc/nitroterm-terminal/main/scripts/uninstall.sh | bash
```
