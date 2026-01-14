use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use cargo_parallels::app::App;
use cargo_parallels::tui::{Renderer, handle_key};

/// Default maximum buffer lines per command
const DEFAULT_MAX_BUFFER_LINES: usize = 10000;

/// Poll interval for command output (milliseconds)
const POLL_INTERVAL_MS: u64 = 10;

#[derive(Parser, Debug)]
#[command(
    name = "cargo-p",
    author,
    version,
    about = "Run multiple commands in parallel with TUI",
    long_about = None
)]
struct Args {
    /// Commands to run in parallel
    #[arg(required = true)]
    commands: Vec<String>,

    /// Maximum buffer lines per command
    #[arg(short = 'b', long, default_value_t = DEFAULT_MAX_BUFFER_LINES)]
    max_buffer_lines: usize,
}

/// Initialize the terminal for TUI
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore the terminal to its original state
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// Run the application
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    // Spawn all commands
    app.spawn_commands().await;

    loop {
        // Update visible lines based on terminal size
        let size = terminal.size()?;
        let visible_lines = size.height.saturating_sub(5) as usize; // Account for borders and status bar
        app.tab_manager_mut()
            .current_tab_mut()
            .set_visible_lines(visible_lines);

        // Render
        terminal.draw(|frame| {
            Renderer::render(frame, &app);
        })?;

        // Poll for command output
        app.poll_commands().await;

        // Handle key events
        if event::poll(Duration::from_millis(POLL_INTERVAL_MS))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key(&mut app, key);
        }

        // Check if we should quit
        if app.should_quit() {
            app.kill_all().await;
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    // Validate commands
    if args.commands.is_empty() {
        eprintln!("Error: At least one command is required");
        std::process::exit(1);
    }

    // Create app
    let app = App::new(args.commands, args.max_buffer_lines);

    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Run application
    let result = run_app(&mut terminal, app).await;

    // Restore terminal
    restore_terminal(&mut terminal)?;

    result
}
