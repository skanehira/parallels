use crate::buffer::OutputLine;

/// Event from background command tasks
pub enum AppEvent {
    /// Output line for a specific tab
    Output { tab_index: usize, line: OutputLine },
    /// Command exited
    Exited { tab_index: usize, exit_code: i32 },
    /// Command failed to start
    Failed { tab_index: usize, reason: String },
}
