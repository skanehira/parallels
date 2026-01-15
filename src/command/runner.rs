use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::buffer::{OutputKind, OutputLine};
use crate::event::AppEvent;

/// Command execution manager
pub struct CommandRunner;

impl CommandRunner {
    /// Spawn a shell command
    ///
    /// Executes the command using `sh -c "command"` format,
    /// capturing stdout/stderr asynchronously.
    ///
    /// The command is spawned in a new process group so that
    /// all child processes can be killed together.
    ///
    /// Events are sent directly to the provided channel.
    pub async fn spawn(
        event_tx: mpsc::Sender<AppEvent>,
        command: &str,
        tab_index: usize,
    ) -> std::io::Result<Child> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Create a new process group with PGID = child PID
            .process_group(0);

        let mut child = cmd.spawn()?;

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let tx = event_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let event = AppEvent::Output {
                        tab_index,
                        line: OutputLine::new(OutputKind::Stdout, line),
                    };
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            });
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let tx = event_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let event = AppEvent::Output {
                        tab_index,
                        line: OutputLine::new(OutputKind::Stderr, line),
                    };
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            });
        }

        Ok(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn command_runner_spawn_returns_error_for_invalid_command() {
        let (tx, _rx) = mpsc::channel(100);
        // sh -c will still succeed even with invalid command
        // but the command itself will fail
        let result = CommandRunner::spawn(tx, "/nonexistent/command", 0).await;
        // spawn succeeds because sh exists
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn command_runner_captures_stdout() {
        let (tx, mut rx) = mpsc::channel(100);
        let _child = CommandRunner::spawn(tx, "echo hello", 0).await.unwrap();

        let mut found_hello = false;
        while let Some(event) = rx.recv().await {
            let AppEvent::Output { line, .. } = event else {
                continue;
            };
            if line.kind == OutputKind::Stdout && line.plain() == "hello" {
                found_hello = true;
                break;
            }
        }
        assert!(found_hello, "Expected to find 'hello' in stdout");
    }

    #[tokio::test]
    async fn command_runner_captures_stderr() {
        let (tx, mut rx) = mpsc::channel(100);
        let _child = CommandRunner::spawn(tx, "echo error >&2", 0).await.unwrap();

        let mut found_error = false;
        while let Some(event) = rx.recv().await {
            let AppEvent::Output { line, .. } = event else {
                continue;
            };
            if line.kind == OutputKind::Stderr && line.plain() == "error" {
                found_error = true;
                break;
            }
        }
        assert!(found_error, "Expected to find 'error' in stderr");
    }

    #[tokio::test]
    async fn command_runner_captures_multiple_lines() {
        let (tx, mut rx) = mpsc::channel(100);
        let _child = CommandRunner::spawn(tx, "echo line1; echo line2; echo line3", 0)
            .await
            .unwrap();

        let mut lines = Vec::new();
        while let Some(event) = rx.recv().await {
            let AppEvent::Output { line, .. } = event else {
                continue;
            };
            if line.kind == OutputKind::Stdout {
                lines.push(line.plain());
            }
        }
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[tokio::test]
    async fn command_runner_child_has_pid() {
        let (tx, _rx) = mpsc::channel(100);
        let child = CommandRunner::spawn(tx, "sleep 0.1", 0).await.unwrap();
        assert!(child.id().is_some());
    }

    #[tokio::test]
    async fn command_runner_child_can_be_killed() {
        let (tx, _rx) = mpsc::channel(100);
        let mut child = CommandRunner::spawn(tx, "sleep 10", 0).await.unwrap();
        let pid = child.id();
        assert!(pid.is_some());

        let result = child.kill().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn command_runner_child_wait_returns_exit_code_success() {
        let (tx, _rx) = mpsc::channel(100);
        let mut child = CommandRunner::spawn(tx, "exit 0", 0).await.unwrap();
        let status = child.wait().await.unwrap();
        assert_eq!(status.code(), Some(0));
    }

    #[tokio::test]
    async fn command_runner_child_wait_returns_exit_code_failure() {
        let (tx, _rx) = mpsc::channel(100);
        let mut child = CommandRunner::spawn(tx, "exit 42", 0).await.unwrap();
        let status = child.wait().await.unwrap();
        assert_eq!(status.code(), Some(42));
    }
}
