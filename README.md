# 🚀 NitroKit Terminal

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

NitroKit is a powerful terminal application written in Rust that provides comprehensive project management functionalities. It automatically generates detailed release notes from git history and intelligently manages project dependencies across multiple programming languages and package managers.

```
███╗   ██╗██╗████████╗██████╗  ██████╗ ██╗  ██╗██╗████████╗
████╗  ██║██║╚══██╔══╝██╔══██╗██╔═══██╗██║ ██╔╝██║╚══██╔══╝
██╔██╗ ██║██║   ██║   ██████╔╝██║   ██║█████╔╝ ██║   ██║   
██║╚██╗██║██║   ██║   ██╔══██╗██║   ██║██╔═██╗ ██║   ██║   
██║ ╚████║██║   ██║   ██║  ██║╚██████╔╝██║  ██╗██║   ██║   
╚═╝  ╚═══╝╚═╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝   ╚═╝   
```

## ✨ Features

### 📋 Release Notes Generation

- **Smart Git Analysis**: Automatically analyzes git commit history
- **Conventional Commits**: Supports conventional commit format (feat, fix, docs, etc.)
- **Categorized Output**: Groups commits by type (features, fixes, breaking changes)
- **Multiple Tag Formats**: Handles various version tag formats (v1.0.0, 1.0.0, etc.)
- **Contributor Statistics**: Includes detailed contributor information
- **Repository Integration**: Generates links for GitHub, GitLab, and Bitbucket
- **Markdown Export**: Creates beautifully formatted markdown files

### 🔄 Dependency Management

- **Multi-Language Support**: 
  - 📦 **Node.js** (npm, yarn, pnpm)
  - 🦀 **Rust** (Cargo)
  - 🐍 **Python** (pip, requirements.txt)
  - 🐘 **PHP** (Composer)
- **Smart Detection**: Automatically detects project types and package managers
- **Backup & Restore**: Creates backups before making changes
- **Security Auditing**: Runs security checks on dependencies
- **Update Verification**: Ensures updates don't break your project
- **Detailed Reporting**: Provides comprehensive update summaries

### 🎯 Interactive Mode

- **User-Friendly Menu**: Easy-to-use interactive interface
- **Command Validation**: Input validation and error handling
- **Progress Indicators**: Visual feedback for long-running operations
- **Colored Output**: Beautiful, colored terminal output

## 🛠️ Getting Started

### Prerequisites

- **Rust** 1.70 or later
- **Git** (for release notes generation)
- **Package Managers** (optional, based on your project type):
  - Node.js with npm/yarn/pnpm
  - Python with pip
  - PHP with Composer

### Installation

#### From Source

```bash
# Clone the repository
git clone https://github.com/mustafagenc/nitrokit-terminal.git
cd nitrokit-terminal

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

#### From Releases

Download the latest binary from [Releases](https://github.com/mustafagenc/nitrokit-terminal/releases) page.

## 🚀 Usage

### Command Line Interface

```bash
# Interactive mode (default)
nitrokit

# Generate release notes
nitrokit release-notes

# Update dependencies
nitrokit update-dependencies

# Show help
nitrokit --help
```

### Interactive Mode

Launch interactive mode for a user-friendly experience:

```bash
nitrokit
```

This will present you with a menu:

```
Available commands:
  1. release-notes        Generate release notes from git commits
  2. update-dependencies  Analyze and update project dependencies
  3. help                 Show this help menu
  4. exit                 Exit Nitrokit

nitrokit> 
```

### Examples

#### Release Notes Generation

```bash
# Generate release notes for current repository
cd nitrokit-terminal
nitrokit release-notes
```

**Output Example:**
```markdown
# Release Notes v1.2.0

## 🚀 Features
- feat: add user authentication system
- feat: implement dark mode support

## 🐛 Bug Fixes  
- fix: resolve memory leak in parser
- fix: handle edge case in validation

## 📚 Documentation
- docs: update API documentation
- docs: add installation guide
```

#### Dependency Updates

```bash
# Update all dependencies in current project
cd nitrokit-terminal
nitrokit update-dependencies
```

**Sample Output:**
```
[INFO] Scanning for dependency files...
[INFO] Found: package.json, Cargo.toml, requirements.txt

📦 Node.js Dependencies:
[INFO] Using package manager: pnpm
[SUCCESS] Updated 5 dependencies
[INFO] Security audit: No vulnerabilities found

🦀 Rust Dependencies:
[SUCCESS] Updated 3 dependencies
[INFO] All dependencies are up to date

🐍 Python Dependencies:
[WARNING] pip not found, skipping Python updates
```

## 🏗️ Project Structure

```
nitrokit-terminal/
├── src/
│   ├── commands/           # Command implementations
│   │   ├── dependency_update.rs
│   │   ├── release_notes.rs
│   │   └── mod.rs
│   ├── utils/              # Utility functions
│   │   ├── file_system.rs
│   │   ├── formatting.rs
│   │   ├── git.rs
│   │   ├── logging.rs
│   │   └── mod.rs
│   ├── tests/              # Test modules
│   │   ├── dependency_update_test.rs
│   │   ├── release_notes_test.rs
│   │   └── mod.rs
│   └── main.rs             # Application entry point
├── .github/
│   └── workflows/          # CI/CD workflows
│       └── rust.yml
├── Cargo.toml              # Rust dependencies
├── README.md
└── LICENSE
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test dependency_update_test

# Run tests with coverage
cargo test --all-features
```

## 🚀 Development

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Generate documentation
cargo doc --open
```

## 📦 Supported Package Managers

| Language | Package Manager | Status | Features |
|----------|----------------|--------|----------|
| Node.js  | npm           | ✅     | Update, audit, backup |
| Node.js  | yarn          | ✅     | Update, audit, backup |
| Node.js  | pnpm          | ✅     | Update, audit, backup |
| Rust     | Cargo         | ✅     | Update, backup |
| Python   | pip           | ✅     | Update from requirements.txt |
| PHP      | Composer      | ✅     | Update, backup |

## 🔧 Configuration

NitroKit works out of the box without configuration, but you can customize behavior through:

- **Environment Variables**: `RUST_LOG=debug` for verbose logging
- **Git Configuration**: Uses your existing git setup
- **Package Manager Settings**: Respects your existing package manager configurations

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'feat: add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Follow [Conventional Commits](https://www.conventionalcommits.org/)
- Add tests for new features
- Update documentation as needed
- Ensure all tests pass: `cargo test`
- Format code: `cargo fmt`
- Run clippy: `cargo clippy`

## 📈 Roadmap

- [ ] **GUI Version**: Desktop application with native UI
- [ ] **More Languages**: Go, Java, C# support
- [ ] **Cloud Integration**: GitHub/GitLab API integration
- [ ] **Template System**: Customizable release note templates
- [ ] **Plugin System**: Extensible architecture for custom commands

## 🐛 Issue Reporting

Found a bug? Please [open an issue](https://github.com/mustafagenc/nitrokit-terminal/issues) with:

- **Environment**: OS, Rust version, etc.
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happened
- **Steps to Reproduce**: How to reproduce the issue
- **Additional Context**: Screenshots, logs, etc.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Clap](https://crates.io/crates/clap) - Command line argument parsing
- [Git2](https://crates.io/crates/git2) - Git repository interaction
- [Colored](https://crates.io/crates/colored) - Terminal color support
- [Serde](https://crates.io/crates/serde) - Serialization framework
- [Tokio](https://crates.io/crates/tokio) - Async runtime

---

⭐ **Star this repository if you find it helpful!**
