use crate::tui::tab::Tab;

/// Multiple tab manager
pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
}

impl TabManager {
    /// Create TabManager from command list
    pub fn new(commands: Vec<String>, max_buffer_lines: usize) -> Self {
        let tabs = commands
            .into_iter()
            .map(|cmd| Tab::new(cmd, max_buffer_lines))
            .collect();
        Self {
            tabs,
            active_index: 0,
        }
    }

    /// Get tab count
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Check if tabs are empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Get active tab index
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// Switch to next tab (wrapping)
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = (self.active_index + 1) % self.tabs.len();
        }
    }

    /// Switch to previous tab (wrapping)
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = if self.active_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_index - 1
            };
        }
    }

    /// Get reference to current tab
    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }

    /// Get mutable reference to current tab
    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_index]
    }

    /// Get tab by index
    pub fn get_tab(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    /// Get mutable tab by index
    pub fn get_tab_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }

    /// Get iterator over all tabs
    pub fn iter(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }

    /// Get mutable iterator over all tabs
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tab> {
        self.tabs.iter_mut()
    }
}
