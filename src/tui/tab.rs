use crate::buffer::{OutputBuffer, OutputLine};

/// Command execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandStatus {
    /// Running
    Running,
    /// Finished with exit code
    Finished { exit_code: i32 },
    /// Failed to start
    Failed { reason: String },
}

/// Maximum characters for tab name display
const MAX_TAB_NAME_LEN: usize = 20;

/// Tab structure representing a command and its output
pub struct Tab {
    command: String,
    buffer: OutputBuffer,
    status: CommandStatus,
    scroll_offset: usize,
    horizontal_scroll: usize,
    auto_scroll: bool,
    visible_lines: usize,
}

impl Tab {
    /// Create a new tab
    pub fn new(command: String, max_buffer_lines: usize) -> Self {
        Self {
            command,
            buffer: OutputBuffer::new(max_buffer_lines),
            status: CommandStatus::Running,
            scroll_offset: 0,
            horizontal_scroll: 0,
            auto_scroll: true,
            visible_lines: 0,
        }
    }

    /// Get the command string
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Get truncated command name for tab display
    pub fn display_name(&self) -> String {
        if self.command.len() <= MAX_TAB_NAME_LEN {
            self.command.clone()
        } else {
            format!("{}...", &self.command[..MAX_TAB_NAME_LEN])
        }
    }

    /// Get command status
    pub fn status(&self) -> &CommandStatus {
        &self.status
    }

    /// Set command status
    pub fn set_status(&mut self, status: CommandStatus) {
        self.status = status;
    }

    /// Add an output line
    pub fn push_output(&mut self, line: OutputLine) {
        self.buffer.push(line);
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Get reference to output buffer
    pub fn buffer(&self) -> &OutputBuffer {
        &self.buffer
    }

    /// Set the number of visible lines
    pub fn set_visible_lines(&mut self, lines: usize) {
        self.visible_lines = lines;
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        let max_offset = self.max_scroll_offset();
        if self.scroll_offset < max_offset {
            self.scroll_offset += 1;
        }
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by half page
    pub fn scroll_half_page_down(&mut self) {
        let half_page = self.visible_lines / 2;
        let max_offset = self.max_scroll_offset();
        self.scroll_offset = (self.scroll_offset + half_page).min(max_offset);
    }

    /// Scroll up by half page
    pub fn scroll_half_page_up(&mut self) {
        let half_page = self.visible_lines / 2;
        self.scroll_offset = self.scroll_offset.saturating_sub(half_page);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// Scroll to specified line
    pub fn scroll_to_line(&mut self, line: usize) {
        let max_offset = self.max_scroll_offset();
        self.scroll_offset = line.min(max_offset);
    }

    /// Check if auto scroll is enabled
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Toggle auto scroll
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }

    /// Set auto scroll
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }

    /// Get current horizontal scroll offset
    pub fn horizontal_scroll(&self) -> usize {
        self.horizontal_scroll
    }

    /// Scroll left by one character
    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
    }

    /// Scroll right by one character
    pub fn scroll_right(&mut self) {
        self.horizontal_scroll += 1;
    }

    /// Scroll to leftmost position
    pub fn scroll_to_left(&mut self) {
        self.horizontal_scroll = 0;
    }

    /// Reset the tab to initial state
    ///
    /// Clears the buffer, resets status to Running, and resets scroll positions.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.status = CommandStatus::Running;
        self.scroll_offset = 0;
        self.horizontal_scroll = 0;
        self.auto_scroll = true;
    }

    /// Calculate maximum scroll offset
    fn max_scroll_offset(&self) -> usize {
        self.buffer.len().saturating_sub(self.visible_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::OutputKind;
    use rstest::rstest;

    #[test]
    fn tab_new_returns_running_status() {
        let tab = Tab::new("cargo build".into(), 100);
        assert_eq!(tab.command(), "cargo build");
        assert_eq!(tab.status(), &CommandStatus::Running);
        assert!(tab.auto_scroll());
    }

    #[rstest]
    #[case("cargo build", "cargo build")]
    #[case("cargo build --release --features foo bar", "cargo build --releas...")]
    fn tab_display_name_returns_correct_name(#[case] command: &str, #[case] expected: &str) {
        let tab = Tab::new(command.into(), 100);
        assert_eq!(tab.display_name(), expected);
    }

    #[test]
    fn tab_scroll_down_increases_offset() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        for i in 0..20 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_top();

        tab.scroll_down();
        assert_eq!(tab.scroll_offset(), 1);

        tab.scroll_down();
        assert_eq!(tab.scroll_offset(), 2);
    }

    #[test]
    fn tab_scroll_down_stops_at_max() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        for i in 0..10 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_top();

        for _ in 0..20 {
            tab.scroll_down();
        }
        // max_offset = 10 - 5 = 5
        assert_eq!(tab.scroll_offset(), 5);
    }

    #[test]
    fn tab_scroll_up_decreases_offset() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        for i in 0..20 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_line(10);

        tab.scroll_up();
        assert_eq!(tab.scroll_offset(), 9);
    }

    #[test]
    fn tab_scroll_up_stops_at_zero() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        for i in 0..10 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_top();

        tab.scroll_up();
        assert_eq!(tab.scroll_offset(), 0);
    }

    #[test]
    fn tab_scroll_half_page_down_moves_by_half_visible_lines() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(10);
        for i in 0..50 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_top();

        tab.scroll_half_page_down();
        assert_eq!(tab.scroll_offset(), 5);

        tab.scroll_half_page_down();
        assert_eq!(tab.scroll_offset(), 10);
    }

    #[test]
    fn tab_scroll_half_page_up_moves_by_half_visible_lines() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(10);
        for i in 0..50 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_line(20);

        tab.scroll_half_page_up();
        assert_eq!(tab.scroll_offset(), 15);

        tab.scroll_half_page_up();
        assert_eq!(tab.scroll_offset(), 10);
    }

    #[test]
    fn tab_scroll_to_top_sets_offset_to_zero() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        for i in 0..20 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }

        tab.scroll_to_top();
        assert_eq!(tab.scroll_offset(), 0);
    }

    #[test]
    fn tab_scroll_to_bottom_sets_offset_to_max() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        for i in 0..20 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        tab.scroll_to_top();

        tab.scroll_to_bottom();
        assert_eq!(tab.scroll_offset(), 15); // 20 - 5 = 15
    }

    #[test]
    fn tab_toggle_auto_scroll_flips_flag() {
        let mut tab = Tab::new("test".into(), 100);
        assert!(tab.auto_scroll());

        tab.toggle_auto_scroll();
        assert!(!tab.auto_scroll());

        tab.toggle_auto_scroll();
        assert!(tab.auto_scroll());
    }

    #[rstest]
    #[case(true, 15)] // auto_scroll enabled: 20 - 5 = 15
    #[case(false, 0)] // auto_scroll disabled: stays at top
    fn tab_push_output_respects_auto_scroll_setting(
        #[case] auto_scroll: bool,
        #[case] expected_offset: usize,
    ) {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_visible_lines(5);
        tab.set_auto_scroll(auto_scroll);
        tab.scroll_to_top();

        for i in 0..20 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }

        assert_eq!(tab.scroll_offset(), expected_offset);
    }

    #[test]
    fn tab_set_status_updates_status() {
        let mut tab = Tab::new("test".into(), 100);
        tab.set_status(CommandStatus::Finished { exit_code: 0 });
        assert_eq!(tab.status(), &CommandStatus::Finished { exit_code: 0 });

        tab.set_status(CommandStatus::Failed {
            reason: "not found".into(),
        });
        assert_eq!(
            tab.status(),
            &CommandStatus::Failed {
                reason: "not found".into()
            }
        );
    }

    #[test]
    fn tab_scroll_right_increases_horizontal_offset() {
        let mut tab = Tab::new("test".into(), 100);
        assert_eq!(tab.horizontal_scroll(), 0);

        tab.scroll_right();
        assert_eq!(tab.horizontal_scroll(), 1);

        tab.scroll_right();
        assert_eq!(tab.horizontal_scroll(), 2);
    }

    #[test]
    fn tab_scroll_left_decreases_horizontal_offset() {
        let mut tab = Tab::new("test".into(), 100);
        tab.scroll_right();
        tab.scroll_right();
        assert_eq!(tab.horizontal_scroll(), 2);

        tab.scroll_left();
        assert_eq!(tab.horizontal_scroll(), 1);
    }

    #[test]
    fn tab_scroll_left_stops_at_zero() {
        let mut tab = Tab::new("test".into(), 100);
        assert_eq!(tab.horizontal_scroll(), 0);

        tab.scroll_left();
        assert_eq!(tab.horizontal_scroll(), 0);
    }

    #[test]
    fn tab_scroll_to_left_resets_horizontal_offset() {
        let mut tab = Tab::new("test".into(), 100);
        tab.scroll_right();
        tab.scroll_right();
        tab.scroll_right();
        assert_eq!(tab.horizontal_scroll(), 3);

        tab.scroll_to_left();
        assert_eq!(tab.horizontal_scroll(), 0);
    }

    #[test]
    fn tab_reset_clears_buffer_and_resets_state() {
        let mut tab = Tab::new("test".into(), 100);
        // Add some output
        for i in 0..10 {
            tab.push_output(OutputLine::new(OutputKind::Stdout, format!("line{}", i)));
        }
        // Modify state
        tab.set_status(CommandStatus::Finished { exit_code: 0 });
        tab.scroll_to_line(5);
        tab.scroll_right();
        tab.scroll_right();
        tab.set_auto_scroll(false);

        // Reset
        tab.reset();

        // Verify all state is reset
        assert!(tab.buffer().is_empty());
        assert_eq!(tab.status(), &CommandStatus::Running);
        assert_eq!(tab.scroll_offset(), 0);
        assert_eq!(tab.horizontal_scroll(), 0);
        assert!(tab.auto_scroll());
    }
}
