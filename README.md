![GitHub Repo stars](https://img.shields.io/github/stars/skanehira/cargo-parallels?style=social)
![GitHub](https://img.shields.io/github/license/skanehira/cargo-parallels)
![GitHub all releases](https://img.shields.io/github/downloads/skanehira/cargo-parallels/total)
![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/skanehira/cargo-parallels/ci.yaml?branch=main)
![GitHub Release Status](https://img.shields.io/github/v/release/skanehira/cargo-parallels)

# cargo-parallels

A TUI tool to run multiple commands in parallel and view their output in real-time.

## Features

- Run multiple commands concurrently
- View stdout/stderr output in real-time with ANSI color support
- Tab-based interface for switching between command outputs
- Vim-like keybindings for navigation
- Search with smartcase (case-insensitive by default, case-sensitive when query contains uppercase)
- Emacs-like keybindings in search mode (Ctrl+W, Ctrl+U, Ctrl+H, etc.)

## Installation

```bash
cargo install cargo-parallels
```

## Usage

```bash
# Run multiple commands
parallels "command1" "command2" "command3"

# Example: Monitor multiple log files
parallels "tail -f /var/log/syslog" "tail -f /var/log/auth.log"

# Example: Run multiple dev servers
parallels "npm run dev" "cargo watch -x run" "docker-compose logs -f"

# Set maximum buffer lines per command (default: 10000)
parallels -b 5000 "command1" "command2"
```

## Keybindings

### Normal Mode

| Key                 | Action                                             |
| ------------------- | -------------------------------------------------- |
| `q`                 | Quit                                               |
| `h` / `l`           | Switch to previous/next tab                        |
| `j` / `k`           | Scroll down/up                                     |
| `Ctrl+d` / `Ctrl+u` | Scroll half page down/up                           |
| `g` / `G`           | Jump to top/bottom                                 |
| `f`                 | Toggle auto-scroll                                 |
| `/`                 | Enter search mode                                  |
| `n` / `N`           | Next/previous search match (when search is active) |

### Search Mode

| Key                 | Action                                   |
| ------------------- | ---------------------------------------- |
| `Enter`             | Confirm search and return to normal mode |
| `Esc`               | Cancel search and return to normal mode  |
| `Ctrl+W`            | Delete word                              |
| `Ctrl+U`            | Clear line                               |
| `Ctrl+H`            | Delete character                         |
| `Ctrl+A` / `Ctrl+E` | Move to start/end of line                |

### Search Behavior

- **Smartcase**: If your search query contains only lowercase letters, the search is case-insensitive. If it contains any uppercase letter, the search becomes case-sensitive.
  - `error` matches "error", "Error", "ERROR"
  - `Error` matches only "Error"

## Contributing

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for an overview of the codebase architecture.

## License

MIT
