# NitroKit

NitroKit is a terminal application written in Rust that provides functionalities for generating release notes and updating dependencies. It is designed to work seamlessly on both Windows and macOS.

## Features

- **Release Notes Generation**: Automatically generates formatted release notes based on the current state of the application.
- **Dependency Update**: Checks for outdated dependencies and updates them accordingly.

## Getting Started

### Prerequisites

- Rust (1.50 or later)
- Cargo (Rust's package manager)

### Installation

- Clone the repository:

```bash
git clone https://github.com/yourusername/nitrokit-terminal.git
cd nitrokit
```

- Build the project:

```bash
cargo build --release
```

- Run the application:

```bash
cargo run --release
```

## Usage

To generate release notes, use the following command:

```bash
nitrokit release-notes
```

To update dependencies, use:

```bash
nitrokit update-dependencies
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
