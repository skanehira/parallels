# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build & Run
```bash
# Build
cargo build

# Release build
cargo build --release

# Run (example)
cargo run -- "echo hello" "sleep 1 && echo world"
```

### Testing
```bash
# Run all tests
cargo test

# Fast test execution with cargo-nextest (recommended)
cargo nextest run

# Run a single test
cargo test test_name

# Generate coverage (requires cargo-llvm-cov)
cargo llvm-cov nextest --lcov --output-path lcov.info
```

### Quality Checks
```bash
# Format check
cargo fmt -- --check

# Apply formatting
cargo fmt

# Static analysis with Clippy
cargo clippy
```

## Architecture

For detailed architecture documentation, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

### Module Overview

```
src/
├── main.rs          # Entry point, event loop, terminal setup
├── lib.rs           # Module re-exports
├── app.rs           # Application state (App struct)
├── event.rs         # Event types for inter-task communication
├── buffer/          # Output buffer management
├── command/         # Command execution (CommandRunner)
├── search/          # Search functionality (smartcase)
└── tui/             # Terminal UI (input, renderer, tabs)
```

### Key Components

- **App** (`src/app.rs`): Central state coordinating tabs, search, and events
- **CommandRunner** (`src/command/runner.rs`): Spawns processes, streams output via mpsc
- **TabManager/Tab** (`src/tui/`): Per-command output buffers and scroll state
- **SearchState** (`src/search/searcher.rs`): Smartcase search with match navigation
- **Renderer** (`src/tui/renderer.rs`): ratatui-based UI rendering

### Data Flow

1. `CommandRunner` spawns processes as tokio tasks
2. Output lines are sent via `mpsc::channel` as `AppEvent::Output`
3. `App::handle_app_event` dispatches to appropriate `Tab`
4. `Renderer` draws current state at ~60fps

### CI/CD Configuration
- **ci.yaml**: Main CI workflow (fmt, clippy, build, test, coverage)
- **audit.yaml**: Security audit for dependencies
- **release.yaml**: Automated release on tag push

### Key Settings
- **Rust version**: Fixed to 1.87 in `rust-toolchain.toml`
- **Edition**: Uses Rust 2024 edition
- **Test tools**: cargo-nextest and cargo-llvm-cov recommended
