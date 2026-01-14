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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_manager_new_creates_tabs_from_commands() {
        let commands = vec!["cmd1".into(), "cmd2".into(), "cmd3".into()];
        let manager = TabManager::new(commands, 100);

        assert_eq!(manager.len(), 3);
        assert!(!manager.is_empty());
        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn tab_manager_new_with_empty_commands() {
        let commands: Vec<String> = vec![];
        let manager = TabManager::new(commands, 100);

        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn tab_manager_next_tab_advances_index() {
        let commands = vec!["cmd1".into(), "cmd2".into(), "cmd3".into()];
        let mut manager = TabManager::new(commands, 100);

        assert_eq!(manager.active_index(), 0);

        manager.next_tab();
        assert_eq!(manager.active_index(), 1);

        manager.next_tab();
        assert_eq!(manager.active_index(), 2);
    }

    #[test]
    fn tab_manager_next_tab_wraps_to_first() {
        let commands = vec!["cmd1".into(), "cmd2".into(), "cmd3".into()];
        let mut manager = TabManager::new(commands, 100);

        manager.next_tab(); // 0 -> 1
        manager.next_tab(); // 1 -> 2
        manager.next_tab(); // 2 -> 0 (wrap)

        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn tab_manager_prev_tab_decreases_index() {
        let commands = vec!["cmd1".into(), "cmd2".into(), "cmd3".into()];
        let mut manager = TabManager::new(commands, 100);

        manager.next_tab();
        manager.next_tab();
        assert_eq!(manager.active_index(), 2);

        manager.prev_tab();
        assert_eq!(manager.active_index(), 1);

        manager.prev_tab();
        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn tab_manager_prev_tab_wraps_to_last() {
        let commands = vec!["cmd1".into(), "cmd2".into(), "cmd3".into()];
        let mut manager = TabManager::new(commands, 100);

        manager.prev_tab(); // 0 -> 2 (wrap)

        assert_eq!(manager.active_index(), 2);
    }

    #[test]
    fn tab_manager_current_tab_returns_active_tab() {
        let commands = vec!["cmd1".into(), "cmd2".into()];
        let manager = TabManager::new(commands, 100);

        assert_eq!(manager.current_tab().command(), "cmd1");
    }

    #[test]
    fn tab_manager_current_tab_mut_allows_modification() {
        let commands = vec!["cmd1".into(), "cmd2".into()];
        let mut manager = TabManager::new(commands, 100);

        manager.current_tab_mut().set_visible_lines(10);
        // Verify modification was applied (indirectly through scroll behavior)
        assert_eq!(manager.current_tab().scroll_offset(), 0);
    }

    #[test]
    fn tab_manager_get_tab_returns_tab_at_index() {
        let commands = vec!["cmd1".into(), "cmd2".into(), "cmd3".into()];
        let manager = TabManager::new(commands, 100);

        assert_eq!(manager.get_tab(0).unwrap().command(), "cmd1");
        assert_eq!(manager.get_tab(1).unwrap().command(), "cmd2");
        assert_eq!(manager.get_tab(2).unwrap().command(), "cmd3");
        assert!(manager.get_tab(3).is_none());
    }

    #[test]
    fn tab_manager_iter_returns_all_tabs() {
        let commands = vec!["cmd1".into(), "cmd2".into()];
        let manager = TabManager::new(commands, 100);

        let tab_commands: Vec<_> = manager.iter().map(|t| t.command()).collect();
        assert_eq!(tab_commands, vec!["cmd1", "cmd2"]);
    }

    #[test]
    fn tab_manager_next_prev_on_empty_does_nothing() {
        let commands: Vec<String> = vec![];
        let mut manager = TabManager::new(commands, 100);

        manager.next_tab();
        assert_eq!(manager.active_index(), 0);

        manager.prev_tab();
        assert_eq!(manager.active_index(), 0);
    }
}
