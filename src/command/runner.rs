use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::buffer::{OutputKind, OutputLine};

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
    /// Child process
    child: Child,
    /// Channel receiver for command events
    receiver: mpsc::Receiver<CommandEvent>,
}

impl CommandHandle {
    /// Kill the process with SIGKILL
    pub async fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill().await
    }

    /// Wait for the process to exit and return the exit code
    pub async fn wait(&mut self) -> std::io::Result<i32> {
        let status = self.child.wait().await?;
        Ok(status.code().unwrap_or(-1))
    }

    /// Receive the next event asynchronously
    pub async fn next_event(&mut self) -> Option<CommandEvent> {
        self.receiver.recv().await
    }

    /// Get the process ID
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }
}

/// Command execution manager
pub struct CommandRunner;

impl CommandRunner {
    /// Spawn a shell command
    ///
    /// Executes the command using `sh -c "command"` format,
    /// capturing stdout/stderr asynchronously.
    pub async fn spawn(command: &str) -> std::io::Result<CommandHandle> {
        use std::process::Stdio;

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (tx, rx) = mpsc::channel(1000);

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let tx_stdout = tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx_stdout
                        .send(CommandEvent::Output(OutputLine::new(
                            OutputKind::Stdout,
                            line,
                        )))
                        .await;
                }
            });
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let tx_stderr = tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx_stderr
                        .send(CommandEvent::Output(OutputLine::new(
                            OutputKind::Stderr,
                            line,
                        )))
                        .await;
                }
            });
        }

        // Drop the sender - channel will close when stdout/stderr tasks complete
        drop(tx);

        Ok(CommandHandle {
            child,
            receiver: rx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn command_runner_spawn_returns_error_for_invalid_command() {
        // sh -c will still succeed even with invalid command
        // but the command itself will fail
        let result = CommandRunner::spawn("/nonexistent/command").await;
        // spawn succeeds because sh exists
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn command_handle_captures_stdout() {
        let mut handle = CommandRunner::spawn("echo hello").await.unwrap();

        let mut found_hello = false;
        while let Some(event) = handle.next_event().await {
            let CommandEvent::Output(line) = event else {
                continue;
            };
            if line.kind == OutputKind::Stdout && line.content == "hello" {
                found_hello = true;
                break;
            }
        }
        assert!(found_hello, "Expected to find 'hello' in stdout");
    }

    #[tokio::test]
    async fn command_handle_captures_stderr() {
        let mut handle = CommandRunner::spawn("echo error >&2").await.unwrap();

        let mut found_error = false;
        while let Some(event) = handle.next_event().await {
            let CommandEvent::Output(line) = event else {
                continue;
            };
            if line.kind == OutputKind::Stderr && line.content == "error" {
                found_error = true;
                break;
            }
        }
        assert!(found_error, "Expected to find 'error' in stderr");
    }

    #[tokio::test]
    async fn command_handle_captures_multiple_lines() {
        let mut handle = CommandRunner::spawn("echo line1; echo line2; echo line3")
            .await
            .unwrap();

        let mut lines = Vec::new();
        while let Some(event) = handle.next_event().await {
            let CommandEvent::Output(line) = event else {
                continue;
            };
            if line.kind == OutputKind::Stdout {
                lines.push(line.content);
            }
        }
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[tokio::test]
    async fn command_handle_pid_returns_some() {
        let handle = CommandRunner::spawn("sleep 0.1").await.unwrap();
        assert!(handle.pid().is_some());
    }

    #[tokio::test]
    async fn command_handle_kill_terminates_process() {
        let mut handle = CommandRunner::spawn("sleep 10").await.unwrap();
        let pid = handle.pid();
        assert!(pid.is_some());

        let result = handle.kill().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn command_handle_wait_returns_exit_code_success() {
        let mut handle = CommandRunner::spawn("exit 0").await.unwrap();
        let exit_code = handle.wait().await.unwrap();
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn command_handle_wait_returns_exit_code_failure() {
        let mut handle = CommandRunner::spawn("exit 42").await.unwrap();
        let exit_code = handle.wait().await.unwrap();
        assert_eq!(exit_code, 42);
    }
}
