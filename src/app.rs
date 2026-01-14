use crate::search::SearchState;
use crate::tui::TabManager;

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
}

impl App {
    /// Initialize the application
    pub fn new(commands: Vec<String>, max_buffer_lines: usize) -> Self {
        Self {
            tab_manager: TabManager::new(commands, max_buffer_lines),
            mode: Mode::Normal,
            search_state: SearchState::new(),
            should_quit: false,
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
