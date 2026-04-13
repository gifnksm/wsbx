use std::{fmt, str::FromStr};

use serde::Deserialize;
use uuid::Uuid;

use crate::SandboxError;

/// The identifier of a Windows Sandbox environment.
///
/// This corresponds to the GUID value used by the `wsb` CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct SandboxId(Uuid);

impl From<Uuid> for SandboxId {
    fn from(value: Uuid) -> Self {
        SandboxId(value)
    }
}

impl FromStr for SandboxId {
    type Err = SandboxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::from_str(s)?;
        Ok(Self(id))
    }
}

impl SandboxId {
    /// Creates a sandbox ID from a UUID value.
    #[must_use]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for SandboxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The user context used when executing a command in a sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunAs {
    /// Runs the command in the currently active user session.
    ExistingLogin,
    /// Runs the command in the system context.
    System,
}
