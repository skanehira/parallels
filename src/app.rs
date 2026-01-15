use std::collections::HashMap;

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use tokio::process::Child;
use tokio::sync::mpsc;

use crate::command::CommandRunner;
use crate::event::AppEvent;
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
    /// Receiver for events from background tasks
    event_rx: mpsc::Receiver<AppEvent>,
    /// Sender for events (kept to clone for spawned tasks)
    event_tx: mpsc::Sender<AppEvent>,
    /// Child processes indexed by tab index
    children: HashMap<usize, Child>,
    /// Pending restart request (tab index)
    pending_restart: Option<usize>,
}

impl App {
    /// Initialize the application
    pub fn new(commands: Vec<String>, max_buffer_lines: usize) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        Self {
            tab_manager: TabManager::new(commands, max_buffer_lines),
            mode: Mode::Normal,
            search_state: SearchState::new(),
            should_quit: false,
            event_rx,
            event_tx,
            children: HashMap::new(),
            pending_restart: None,
        }
    }

    /// Spawn all commands asynchronously with background output processing
    pub async fn spawn_commands(&mut self) {
        // Collect commands first to avoid borrow conflict
        let commands: Vec<String> = self
            .tab_manager
            .iter()
            .map(|tab| tab.command().to_string())
            .collect();

        for (tab_index, command) in commands.into_iter().enumerate() {
            let tx = self.event_tx.clone();
            match CommandRunner::spawn(tx.clone(), &command, tab_index).await {
                Ok(child) => {
                    self.children.insert(tab_index, child);
                }
                Err(e) => {
                    let _ = tx
                        .send(AppEvent::Failed {
                            tab_index,
                            reason: e.to_string(),
                        })
                        .await;
                }
            }
        }
    }

    /// Receive an event asynchronously (for use with select!)
    pub async fn recv_event(&mut self) -> Option<AppEvent> {
        self.event_rx.recv().await
    }

    /// Handle a single app event
    pub fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Output { tab_index, line } => {
                if let Some(tab) = self.tab_manager.get_tab_mut(tab_index) {
                    tab.push_output(line);
                }
            }
            AppEvent::Exited {
                tab_index,
                exit_code,
            } => {
                if let Some(tab) = self.tab_manager.get_tab_mut(tab_index) {
                    tab.set_status(CommandStatus::Finished { exit_code });
                }
            }
            AppEvent::Failed { tab_index, reason } => {
                if let Some(tab) = self.tab_manager.get_tab_mut(tab_index) {
                    tab.set_status(CommandStatus::Failed { reason });
                }
            }
        }
    }

    /// Kill all running processes
    ///
    /// Sends SIGKILL to all process groups to ensure child processes
    /// (e.g., servers started by shell commands) are also terminated.
    /// Waits for each process to terminate before returning.
    pub async fn kill_all(&mut self) {
        for child in self.children.values_mut() {
            if let Some(pid) = child.id() {
                // Send SIGKILL to the process group (PGID = PID because we used process_group(0))
                let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
            }
            // Wait for the process to terminate
            let _ = child.wait().await;
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

    /// Request restart for a specific tab
    pub fn request_restart(&mut self, tab_index: usize) {
        self.pending_restart = Some(tab_index);
    }

    /// Take pending restart request
    ///
    /// Returns the tab index if a restart was requested, None otherwise.
    /// Clears the pending restart after taking.
    pub fn take_pending_restart(&mut self) -> Option<usize> {
        self.pending_restart.take()
    }

    /// Restart a specific tab's command
    ///
    /// Kills the existing process, resets the tab state, and spawns a new process.
    pub async fn restart_process(&mut self, tab_index: usize) {
        // Kill existing process if any
        if let Some(mut child) = self.children.remove(&tab_index) {
            if let Some(pid) = child.id() {
                let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
            }
            let _ = child.wait().await;
        }

        // Reset tab state
        if let Some(tab) = self.tab_manager.get_tab_mut(tab_index) {
            tab.reset();
        }

        // Get command for this tab
        let command = self
            .tab_manager
            .get_tab(tab_index)
            .map(|tab| tab.command().to_string());

        // Spawn new process
        if let Some(command) = command {
            let tx = self.event_tx.clone();
            match CommandRunner::spawn(tx.clone(), &command, tab_index).await {
                Ok(child) => {
                    self.children.insert(tab_index, child);
                }
                Err(e) => {
                    let _ = tx
                        .send(AppEvent::Failed {
                            tab_index,
                            reason: e.to_string(),
                        })
                        .await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::sys::signal::kill;

    /// Check if a process exists by sending signal 0
    fn process_exists(pid: i32) -> bool {
        kill(Pid::from_raw(pid), None).is_ok()
    }

    #[test]
    fn app_new_initializes_correctly() {
        let app = App::new(vec!["cmd1".into(), "cmd2".into()], 100);

        assert_eq!(app.tab_manager().len(), 2);
        assert_eq!(app.mode(), Mode::Normal);
        assert!(!app.should_quit());
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
    async fn app_spawn_commands_starts_background_tasks() {
        let mut app = App::new(vec!["echo hello".into()], 100);

        app.spawn_commands().await;

        // Receive and handle events
        let timeout = std::time::Duration::from_millis(500);
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            tokio::select! {
                Some(event) = app.recv_event() => {
                    app.handle_app_event(event);
                    if !app.tab_manager().current_tab().buffer().is_empty() {
                        break;
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
            }
        }

        // Check if output was received
        let buffer = app.tab_manager().current_tab().buffer();
        assert!(!buffer.is_empty(), "Should have received output");
    }

    #[tokio::test]
    async fn app_recv_event_handles_output() {
        let mut app = App::new(vec!["echo test_line".into()], 100);

        app.spawn_commands().await;

        // Receive and handle events
        let timeout = std::time::Duration::from_millis(500);
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            tokio::select! {
                Some(event) = app.recv_event() => {
                    app.handle_app_event(event);
                    if !app.tab_manager().current_tab().buffer().is_empty() {
                        break;
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
            }
        }

        let buffer = app.tab_manager().current_tab().buffer();
        assert!(!buffer.is_empty(), "Should have received output");
    }

    #[tokio::test]
    async fn app_kill_all_terminates_child_processes() {
        // Spawn a command that runs a long-running child process
        // The shell (sh) will spawn sleep as a child process
        let mut app = App::new(vec!["sleep 100".into()], 100);
        app.spawn_commands().await;

        // Wait a bit for process to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Get the shell process PID
        let shell_pid = app
            .children
            .get(&0)
            .expect("Should have child at index 0")
            .id()
            .expect("Should have PID");

        // Verify the process is running
        assert!(
            process_exists(shell_pid as i32),
            "Shell process should be running"
        );

        // Kill all processes and wait for them to terminate
        app.kill_all().await;

        // Verify the shell process is terminated
        assert!(
            !process_exists(shell_pid as i32),
            "Shell process should be terminated after kill_all"
        );
    }

    #[test]
    fn app_request_restart_sets_pending() {
        let mut app = App::new(vec!["cmd".into()], 100);

        // Initially no pending restart
        assert!(app.take_pending_restart().is_none());

        // Request restart for tab 0
        app.request_restart(0);

        // Should have pending restart
        assert_eq!(app.take_pending_restart(), Some(0));

        // After taking, should be None again
        assert!(app.take_pending_restart().is_none());
    }

    #[tokio::test]
    async fn app_restart_tab_kills_and_respawns() {
        use crate::tui::CommandStatus;

        let mut app = App::new(vec!["sleep 100".into()], 100);
        app.spawn_commands().await;

        // Wait a bit for process to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Get the original PID
        let original_pid = app
            .children
            .get(&0)
            .expect("Should have child")
            .id()
            .expect("Should have PID");

        // Verify process is running
        assert!(process_exists(original_pid as i32));

        // Add some output to the tab
        app.tab_manager_mut().get_tab_mut(0).unwrap().push_output(
            crate::buffer::OutputLine::new(crate::buffer::OutputKind::Stdout, "test".into()),
        );
        app.tab_manager_mut()
            .get_tab_mut(0)
            .unwrap()
            .set_status(CommandStatus::Finished { exit_code: 0 });

        // Restart the tab
        app.restart_process(0).await;

        // Original process should be killed
        assert!(
            !process_exists(original_pid as i32),
            "Original process should be terminated"
        );

        // New process should be spawned
        let new_pid = app
            .children
            .get(&0)
            .expect("Should have new child")
            .id()
            .expect("Should have PID");
        assert_ne!(original_pid, new_pid, "New process should have different PID");
        assert!(
            process_exists(new_pid as i32),
            "New process should be running"
        );

        // Tab should be reset
        assert!(app.tab_manager().get_tab(0).unwrap().buffer().is_empty());
        assert_eq!(
            app.tab_manager().get_tab(0).unwrap().status(),
            &CommandStatus::Running
        );

        // Cleanup
        app.kill_all().await;
    }
}
