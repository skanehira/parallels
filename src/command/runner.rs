use crate::buffer::OutputLine;

/// Output event from command
#[derive(Debug)]
pub enum CommandEvent {
    /// Output line
    Output(OutputLine),
    /// Process exited
    Exited { exit_code: i32 },
    /// Error occurred
    Error { message: String },
}

/// Handle for managing a running command
pub struct CommandHandle {
    // To be implemented in phase 5
}

/// Command execution manager
pub struct CommandRunner;

impl CommandRunner {
    /// Spawn a shell command
    pub async fn spawn(_command: &str) -> std::io::Result<CommandHandle> {
        // To be implemented in phase 5
        todo!()
    }
}
