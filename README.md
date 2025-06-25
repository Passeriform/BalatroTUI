# Balatro TUI

A minimal clone of Balatro built in Rust that runs in your terminal.

![Demo Video](assets/demo.gif)

> NOTE: This project is WIP. Please check [TODO](TODO) for feature tracking list.

---

## Features

- Play a game of Balatro in your terminal 🃏
- Fast, minimal, and cross-platform
- Modular codebase with core logic and widgets

## Project Structure

- `balatro_tui/` - Main TUI application
- `balatro_tui_core/` - Game logic and core types
- `balatro_tui_widgets/` - UI widgets for the TUI

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable recommended)

### Build and Run

```sh
cargo run --release --package balatro_tui
```

### Format and Lint

```sh
cargo fmt
cargo clippy
```

### Run Tests

```sh
cargo test --workspace
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

*NOTE: All rights of Balatro the game reside with the original developer LocalThunk.*
