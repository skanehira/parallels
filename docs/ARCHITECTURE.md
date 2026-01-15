# Architecture

This document describes the architecture of `parallels` for contributors.

## Overview

`parallels` is a TUI application that runs multiple shell commands in parallel and displays their output in a tabbed interface. It uses:

- **tokio** for async runtime and process management
- **ratatui** + **crossterm** for terminal UI
- **clap** for CLI argument parsing

## Module Structure

```
src/
├── main.rs          # Entry point, event loop, terminal setup
├── lib.rs           # Module re-exports
├── app.rs           # Application state (App struct)
├── event.rs         # Event types for inter-task communication
├── buffer/          # Output buffer management
│   ├── mod.rs
│   └── output.rs    # OutputBuffer, OutputLine, OutputKind
├── command/         # Command execution
│   ├── mod.rs
│   └── runner.rs    # CommandRunner - spawns processes
├── search/          # Search functionality
│   ├── mod.rs
│   └── searcher.rs  # SearchState, Match - smartcase search
└── tui/             # Terminal UI components
    ├── mod.rs
    ├── input.rs     # Keyboard input handling
    ├── renderer.rs  # UI rendering
    ├── tab.rs       # Tab state (per-command)
    └── tab_manager.rs # Tab collection management
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │ EventStream │    │   App       │    │ Terminal + Renderer │ │
│  │ (keyboard)  │───▶│ (state)     │───▶│ (display)           │ │
│  └─────────────┘    └─────────────┘    └─────────────────────┘ │
│                            ▲                                    │
│                            │ mpsc channel                       │
│                     ┌──────┴──────┐                            │
│                     │  AppEvent   │                            │
│                     └──────┬──────┘                            │
│              ┌─────────────┼─────────────┐                     │
│              ▼             ▼             ▼                     │
│      ┌───────────┐  ┌───────────┐  ┌───────────┐              │
│      │ Command 1 │  │ Command 2 │  │ Command N │              │
│      │ (tokio)   │  │ (tokio)   │  │ (tokio)   │              │
│      └───────────┘  └───────────┘  └───────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### App (`src/app.rs`)

Central application state that coordinates all components:

```rust
pub struct App {
    tab_manager: TabManager,     // Tab collection
    mode: Mode,                  // Normal or Search
    search_state: SearchState,  // Search query and matches
    should_quit: bool,
    event_rx: mpsc::Receiver<AppEvent>,  // Receive from background tasks
    event_tx: mpsc::Sender<AppEvent>,    // Clone for spawned tasks
    children: Vec<Child>,                // Child processes
}
```

### Event Loop (`src/main.rs`)

The main event loop uses `tokio::select!` to handle three event sources concurrently:

1. **App events** - Output from background command tasks
2. **Key events** - User keyboard input
3. **Render timer** - Fixed-interval UI updates (~60fps)

```rust
tokio::select! {
    Some(event) = app.recv_event() => { /* handle output */ }
    Some(Ok(Event::Key(key))) = event_stream.next() => { /* handle input */ }
    _ = render_interval.tick() => { /* render UI */ }
}
```

### CommandRunner (`src/command/runner.rs`)

Spawns shell commands and streams their output:

- Executes commands via `sh -c "command"`
- Spawns separate tokio tasks for stdout and stderr
- Sends `AppEvent::Output` for each line
- Sends `AppEvent::Exited` when process completes

### TabManager / Tab (`src/tui/tab_manager.rs`, `src/tui/tab.rs`)

Manages the collection of command tabs:

- **TabManager**: Collection of tabs, active tab index, navigation
- **Tab**: Per-command state including:
  - Command string
  - OutputBuffer (ring buffer with max lines)
  - Scroll position (vertical and horizontal)
  - Auto-scroll flag
  - Command status (Running/Finished/Failed)

### OutputBuffer (`src/buffer/output.rs`)

Ring buffer for command output:

- Fixed maximum size (configurable via `-b` flag)
- Automatically discards oldest lines when full
- ANSI escape sequence parsing via `ansi-to-tui`
- Pre-parsed spans for efficient rendering

### SearchState (`src/search/searcher.rs`)

Search functionality with smartcase:

- **Smartcase**: lowercase query = case-insensitive, uppercase = case-sensitive
- Stores match positions (line, start byte, length)
- Supports next/previous match navigation
- Uses `tui-input` for Emacs-like text editing

### Renderer (`src/tui/renderer.rs`)

Renders the UI using ratatui:

- Tab bar (top)
- Output area (middle) with search highlighting
- Status bar (bottom) showing mode and keybindings

### Input Handler (`src/tui/input.rs`)

Handles keyboard input based on current mode:

- **Normal mode**: Navigation (h/l for horizontal scroll, j/k for vertical scroll), tab switching (Ctrl-h/l), search initiation
- **Search mode**: Text input with Emacs keybindings via `tui-input`

## Key Design Decisions

### Async Architecture

All command output is handled asynchronously via tokio tasks and mpsc channels. This ensures:
- UI remains responsive during heavy output
- Multiple commands can output simultaneously
- No blocking on slow commands

### ANSI Color Support

ANSI escape sequences are parsed once when output is received (`OutputLine::new`), not during rendering. This improves render performance.

### Smartcase Search

Follows Vim's `smartcase` behavior:
- Query `error` → matches "error", "Error", "ERROR"
- Query `Error` → matches only "Error"

This provides intuitive search without requiring a toggle.

### Ring Buffer

Output is stored in a ring buffer to prevent memory exhaustion from long-running commands. Default limit is 10,000 lines per command.

## Testing

Tests are organized per-module using `#[cfg(test)] mod tests`:

- Unit tests for each component
- Snapshot tests for renderer output (via `insta`)
- Async tests for command execution (via `#[tokio::test]`)

Run tests:
```bash
cargo test
```

## Adding New Features

### Adding a New Keybinding

1. Edit `src/tui/input.rs`
2. Add case to `handle_normal_mode` or `handle_search_mode`
3. Add test in the same file
4. Update status bar hint in `src/tui/renderer.rs` if needed

### Adding a New Event Type

1. Add variant to `AppEvent` in `src/event.rs`
2. Handle in `App::handle_app_event` in `src/app.rs`
3. Send from appropriate location (e.g., `CommandRunner`)

### Modifying Search Behavior

1. Edit `src/search/searcher.rs`
2. Modify `SearchState::search` method
3. Add tests for new behavior
