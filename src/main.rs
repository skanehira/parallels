use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{Event, EventStream, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::time::interval;

use parallels::app::App;
use parallels::tui::{Renderer, handle_key};

/// Default maximum buffer lines per command
const DEFAULT_MAX_BUFFER_LINES: usize = 10000;

/// Render interval (milliseconds)
const RENDER_INTERVAL_MS: u64 = 16; // ~60fps

#[derive(Parser, Debug)]
#[command(
    name = "parallels",
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
    // Spawn all commands (starts background tasks)
    app.spawn_commands().await;

    let mut event_stream = EventStream::new();
    let mut render_interval = interval(Duration::from_millis(RENDER_INTERVAL_MS));

    loop {
        // Update visible lines for all tabs based on terminal size
        let size = terminal.size()?;
        let visible_lines = size.height.saturating_sub(5) as usize;
        for tab in app.tab_manager_mut().iter_mut() {
            tab.set_visible_lines(visible_lines);
        }

        tokio::select! {
            // Handle app events from background command tasks
            Some(event) = app.recv_event() => {
                app.handle_app_event(event);
            }
            // Handle key events
            Some(Ok(Event::Key(key))) = event_stream.next() => {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key);

                    // Handle pending restart request
                    if let Some(tab_index) = app.take_pending_restart() {
                        app.restart_process(tab_index).await;
                    }
                }
            }
            // Render at fixed interval
            _ = render_interval.tick() => {
                terminal.draw(|frame| {
                    Renderer::render(frame, &app);
                })?;
            }
        }

        // Check if we should quit
        if app.should_quit() {
            // Kill all child processes before exiting
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
