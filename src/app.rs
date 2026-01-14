use crate::buffer::OutputLine;
use crate::command::{CommandEvent, CommandHandle, CommandRunner};
use crate::search::SearchState;
use crate::tui::{CommandStatus, TabManager};

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal mode
    Normal,
    /// Search mode
    Search,
}

/// Application state
pub struct App {
    tab_manager: TabManager,
    mode: Mode,
    search_state: SearchState,
    should_quit: bool,
    /// Command handles for running processes
    handles: Vec<Option<CommandHandle>>,
}

impl App {
    /// Initialize the application
    pub fn new(commands: Vec<String>, max_buffer_lines: usize) -> Self {
        let num_commands = commands.len();
        let handles = (0..num_commands).map(|_| None).collect();
        Self {
            tab_manager: TabManager::new(commands, max_buffer_lines),
            mode: Mode::Normal,
            search_state: SearchState::new(),
            should_quit: false,
            handles,
        }
    }

    /// Spawn all commands asynchronously
    pub async fn spawn_commands(&mut self) {
        // Collect commands first to avoid borrow conflict
        let commands: Vec<String> = self
            .tab_manager
            .iter()
            .map(|tab| tab.command().to_string())
            .collect();

        for (i, command) in commands.into_iter().enumerate() {
            match CommandRunner::spawn(&command).await {
                Ok(handle) => {
                    self.handles[i] = Some(handle);
                }
                Err(e) => {
                    if let Some(tab) = self.tab_manager.get_tab_mut(i) {
                        tab.set_status(CommandStatus::Failed {
                            reason: e.to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Poll all command handles for events
    pub async fn poll_commands(&mut self) {
        for (i, handle_opt) in self.handles.iter_mut().enumerate() {
            if let Some(handle) = handle_opt {
                // Try to receive event without blocking
                match tokio::time::timeout(std::time::Duration::from_millis(1), handle.next_event())
                    .await
                {
                    Ok(Some(event)) => {
                        if let Some(tab) = self.tab_manager.get_tab_mut(i) {
                            match event {
                                CommandEvent::Output(line) => {
                                    tab.push_output(line);
                                }
                                CommandEvent::Exited { exit_code } => {
                                    tab.set_status(CommandStatus::Finished { exit_code });
                                }
                                CommandEvent::Error { message } => {
                                    tab.push_output(OutputLine::new(
                                        crate::buffer::OutputKind::Stderr,
                                        format!("[error] {}", message),
                                    ));
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        // Channel closed, process likely exited
                    }
                    Err(_) => {
                        // Timeout, no event available
                    }
                }
            }
        }
    }

    /// Kill all running processes
    pub async fn kill_all(&mut self) {
        for handle_opt in self.handles.iter_mut() {
            if let Some(handle) = handle_opt.take() {
                let mut handle = handle;
                let _ = handle.kill().await;
            }
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Set the quit flag
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get current mode
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Get reference to tab manager
    pub fn tab_manager(&self) -> &TabManager {
        &self.tab_manager
    }

    /// Get mutable reference to tab manager
    pub fn tab_manager_mut(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    /// Get reference to search state
    pub fn search_state(&self) -> &SearchState {
        &self.search_state
    }

    /// Get mutable reference to search state
    pub fn search_state_mut(&mut self) -> &mut SearchState {
        &mut self.search_state
    }

    /// Search in current tab's buffer
    ///
    /// This method is needed to avoid borrow conflicts when accessing
    /// both tab_manager and search_state mutably.
    pub fn search_in_current_tab(&mut self, query: &str) {
        let buffer = self.tab_manager.current_tab().buffer();
        self.search_state.search(query, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_new_initializes_correctly() {
        let app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);

        assert_eq!(app.tab_manager().len(), 2);
        assert_eq!(app.mode(), Mode::Normal);
        assert!(!app.should_quit());
        assert_eq!(app.handles.len(), 2);
    }

    #[test]
    fn app_mode_transition_works() {
        let mut app = App::new(vec!["cmd".into()], 100);

        assert_eq!(app.mode(), Mode::Normal);

        app.set_mode(Mode::Search);
        assert_eq!(app.mode(), Mode::Search);

        app.set_mode(Mode::Normal);
        assert_eq!(app.mode(), Mode::Normal);
    }

    #[test]
    fn app_quit_sets_flag() {
        let mut app = App::new(vec!["cmd".into()], 100);

        assert!(!app.should_quit());

        app.quit();
        assert!(app.should_quit());
    }

    #[tokio::test]
    async fn app_spawn_commands_spawns_processes() {
        let mut app = App::new(vec!["echo hello".into()], 100);

        app.spawn_commands().await;

        // The handle should be set
        assert!(app.handles[0].is_some());
    }

    #[tokio::test]
    async fn app_poll_commands_receives_output() {
        let mut app = App::new(vec!["echo test_output".into()], 100);

        app.spawn_commands().await;

        // Wait a bit for output and poll multiple times
        for _ in 0..50 {
            app.poll_commands().await;
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;

            // Early exit if we got output
            if !app.tab_manager().current_tab().buffer().is_empty() {
                break;
            }
        }

        // Check if output was received
        let buffer = app.tab_manager().current_tab().buffer();
        assert!(!buffer.is_empty(), "Should have received output");
    }

    #[tokio::test]
    async fn app_kill_all_terminates_processes() {
        let mut app = App::new(vec!["sleep 10".into()], 100);

        app.spawn_commands().await;
        assert!(app.handles[0].is_some());

        app.kill_all().await;
        assert!(app.handles[0].is_none());
    }
}
