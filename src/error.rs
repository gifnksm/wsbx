use std::process::ExitStatus;

/// Errors that can occur while interacting with Windows Sandbox through `wsb`.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    /// Failed to parse a sandbox ID from its string representation.
    #[error("failed to parse sandbox ID")]
    ParseSandboxId(#[from] uuid::Error),
    /// Failed to start or communicate with the `wsb` process.
    #[error("failed to execute wsb command")]
    ExecuteWsbCommand(#[source] std::io::Error),
    /// The `wsb` command exited with a non-success status.
    ///
    /// When available, the underlying error source contains the message written
    /// to standard error by `wsb`.
    #[error("wsb command exited with non-successful exit status: {status}")]
    WsbCommand {
        /// The exit status returned by `wsb`.
        status: ExitStatus,
        #[source]
        /// Additional error information reported by `wsb`, if any.
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
    /// Failed to deserialize the JSON response returned by `wsb --raw`.
    #[error("failed to deserialize wsb command response")]
    DeserializeResponse(#[source] serde_json::Error),
}
