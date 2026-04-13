//! Types for constructing Windows Sandbox configuration XML passed to
//! `wsb start --config`.
//!
//! This module follows the Windows Sandbox `.wsb` configuration format
//! documented by Microsoft. For details, see
//! [Use and configure Windows Sandbox](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file).
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use crate::xml::Xml;

/// A three-state option used by Windows Sandbox configuration values.
///
/// This is used for settings that accept `Enable`, `Disable`, or `Default` in
/// the generated configuration XML.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OptionState {
    /// Explicitly enables the option.
    Enable,
    /// Explicitly disables the option.
    Disable,
    /// Uses the Windows Sandbox default behavior for the option.
    #[default]
    Default,
}

impl From<bool> for OptionState {
    fn from(value: bool) -> Self {
        if value {
            OptionState::Enable
        } else {
            OptionState::Disable
        }
    }
}

/// A builder-style representation of Windows Sandbox configuration XML.
///
/// Use this type to construct the XML passed to `wsb start --config`.
///
/// # Examples
///
/// Build a configuration with disabled networking and a mapped folder:
///
/// ```rust
/// use wsbx::config::{MappedFolder, SandboxConfig};
///
/// let config = SandboxConfig::new()
///     .networking(false)
///     .mapped_folder(
///         MappedFolder::new(r"C:\host")
///             .sandbox_folder(r"C:\sandbox")
///             .read_only(true),
///     );
///
/// let xml = config.to_os_string();
/// assert!(!xml.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct SandboxConfig {
    vgpu: Option<OptionState>,
    networking: Option<OptionState>,
    mapped_folders: Vec<MappedFolder>,
    logon_command: Option<Command>,
    audio_input: Option<OptionState>,
    video_input: Option<OptionState>,
    protected_client: Option<OptionState>,
    printer_redirection: Option<OptionState>,
    clipboard_redirection: Option<OptionState>,
    memory_in_mb: Option<u32>,
}

/// A folder mapping from the host into a sandbox.
///
/// # Examples
///
/// ```rust
/// use wsbx::config::MappedFolder;
///
/// let folder = MappedFolder::new(r"C:\host")
///     .sandbox_folder(r"C:\sandbox")
///     .read_only(true);
/// ```
#[derive(Debug, Clone)]
pub struct MappedFolder {
    host_folder: PathBuf,
    sandbox_folder: Option<PathBuf>,
    read_only: Option<bool>,
}

/// A command used in the `<LogonCommand>` section of a sandbox configuration.
///
/// This corresponds to the nested `<Command>` element inside
/// `<LogonCommand>`.
#[derive(Debug, Clone)]
pub struct Command(OsString);

impl SandboxConfig {
    /// Creates an empty sandbox configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `vGPU` option.
    #[must_use]
    pub fn vgpu<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.vgpu = Some(state.into());
        self
    }

    /// Sets the `Networking` option.
    #[must_use]
    pub fn networking<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.networking = Some(state.into());
        self
    }

    /// Adds a mapped folder entry to the configuration.
    #[must_use]
    pub fn mapped_folder<F>(mut self, folder: F) -> Self
    where
        F: Into<MappedFolder>,
    {
        self.mapped_folders.push(folder.into());
        self
    }

    /// Sets the command to run after the sandbox logs on.
    #[must_use]
    pub fn logon_command<C>(mut self, command: C) -> Self
    where
        C: Into<Command>,
    {
        self.logon_command = Some(command.into());
        self
    }

    /// Sets the `AudioInput` option.
    #[must_use]
    pub fn audio_input<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.audio_input = Some(state.into());
        self
    }

    /// Sets the `VideoInput` option.
    #[must_use]
    pub fn video_input<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.video_input = Some(state.into());
        self
    }

    /// Sets the `ProtectedClient` option.
    #[must_use]
    pub fn protected_client<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.protected_client = Some(state.into());
        self
    }

    /// Sets the `PrinterRedirection` option.
    #[must_use]
    pub fn printer_redirection<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.printer_redirection = Some(state.into());
        self
    }

    /// Sets the `ClipboardRedirection` option.
    #[must_use]
    pub fn clipboard_redirection<S>(mut self, state: S) -> Self
    where
        S: Into<OptionState>,
    {
        self.clipboard_redirection = Some(state.into());
        self
    }

    /// Sets the `MemoryInMB` value.
    #[must_use]
    pub fn memory_in_mb(mut self, mb: u32) -> Self {
        self.memory_in_mb = Some(mb);
        self
    }

    /// Serializes the configuration as compact XML.
    #[must_use]
    pub fn to_os_string(&self) -> OsString {
        Xml::from(self).to_os_string()
    }

    /// Serializes the configuration as pretty-printed XML.
    #[must_use]
    pub fn to_pretty_os_string(&self) -> OsString {
        Xml::from(self).to_pretty_os_string()
    }
}

impl MappedFolder {
    /// Creates a mapped folder entry from a host folder path.
    ///
    /// If no sandbox folder is specified, Windows Sandbox uses its default
    /// destination.
    #[must_use]
    pub fn new<F>(host_folder: F) -> Self
    where
        F: Into<PathBuf>,
    {
        Self {
            host_folder: host_folder.into(),
            sandbox_folder: None,
            read_only: None,
        }
    }

    /// Sets the destination folder inside the sandbox.
    #[must_use]
    pub fn sandbox_folder<F>(mut self, folder: F) -> Self
    where
        F: Into<PathBuf>,
    {
        self.sandbox_folder = Some(folder.into());
        self
    }

    /// Sets whether the mapped folder is read-only inside the sandbox.
    #[must_use]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }
}

macro_rules! impl_from_for_mapped_folder {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for MappedFolder {
                fn from(host_folder: $ty) -> Self {
                    MappedFolder::new(host_folder)
                }
            }
        )*
    };
}
impl_from_for_mapped_folder!(PathBuf, &Path, String, &str, OsString, &OsStr);

impl Command {
    /// Creates a command value for use in [`SandboxConfig::logon_command`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wsbx::config::{Command, SandboxConfig};
    ///
    /// let config = SandboxConfig::new().logon_command(Command::new("cmd.exe"));
    /// let xml = config.to_os_string();
    /// assert!(!xml.is_empty());
    /// ```
    #[must_use]
    pub fn new<C>(command: C) -> Self
    where
        C: Into<OsString>,
    {
        Self(command.into())
    }
}

macro_rules! impl_from_for_command {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Command {
                fn from(command: $ty) -> Self {
                    Command::new(command)
                }
            }
        )*
    };
}

impl_from_for_command!(String, &str, OsString, &OsStr);

impl From<&OptionState> for Xml {
    fn from(state: &OptionState) -> Self {
        match state {
            OptionState::Enable => Xml::text("Enable"),
            OptionState::Disable => Xml::text("Disable"),
            OptionState::Default => Xml::text("Default"),
        }
    }
}

impl From<&SandboxConfig> for Xml {
    fn from(config: &SandboxConfig) -> Self {
        let SandboxConfig {
            vgpu,
            networking,
            mapped_folders,
            logon_command,
            audio_input,
            video_input,
            protected_client,
            printer_redirection,
            clipboard_redirection,
            memory_in_mb,
        } = config;

        let mut content = vec![];
        if let Some(vgpu) = vgpu {
            content.push(Xml::element("vGPU", [vgpu]));
        }
        if let Some(networking) = networking {
            content.push(Xml::element("Networking", [networking]));
        }
        if !mapped_folders.is_empty() {
            content.push(Xml::element("MappedFolders", mapped_folders));
        }
        if let Some(logon_command) = logon_command {
            content.push(Xml::element("LogonCommand", [logon_command]));
        }
        if let Some(audio_input) = audio_input {
            content.push(Xml::element("AudioInput", [audio_input]));
        }
        if let Some(video_input) = video_input {
            content.push(Xml::element("VideoInput", [video_input]));
        }
        if let Some(protected_client) = protected_client {
            content.push(Xml::element("ProtectedClient", [protected_client]));
        }
        if let Some(printer_redirection) = printer_redirection {
            content.push(Xml::element("PrinterRedirection", [printer_redirection]));
        }
        if let Some(clipboard_redirection) = clipboard_redirection {
            content.push(Xml::element(
                "ClipboardRedirection",
                [clipboard_redirection],
            ));
        }
        if let Some(memory_in_mb) = memory_in_mb {
            content.push(Xml::element(
                "MemoryInMB",
                [Xml::text(memory_in_mb.to_string())],
            ));
        }

        Xml::element("Configuration", content)
    }
}

impl From<&MappedFolder> for Xml {
    fn from(folder: &MappedFolder) -> Self {
        let MappedFolder {
            host_folder,
            sandbox_folder,
            read_only,
        } = folder;

        let mut content = vec![Xml::element("HostFolder", [Xml::text(host_folder)])];
        if let Some(sandbox_folder) = sandbox_folder {
            content.push(Xml::element("SandboxFolder", [Xml::text(sandbox_folder)]));
        }
        if let Some(read_only) = read_only {
            let text = if *read_only { "true" } else { "false" };
            content.push(Xml::element("ReadOnly", [Xml::text(text)]));
        }

        Xml::element("MappedFolder", content)
    }
}

impl From<&Command> for Xml {
    fn from(command: &Command) -> Self {
        let Command(command) = command;
        Xml::element("Command", [Xml::text(command)])
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn mapped_folder_accepts_path_like_inputs() {
        let config = SandboxConfig::new()
            .mapped_folder(r"C:\host")
            .mapped_folder(Path::new(r"C:\host2"));

        assert_eq!(
            config.to_os_string(),
            concat!(
                r"<Configuration>",
                r"<MappedFolders>",
                r"<MappedFolder>",
                r"<HostFolder>C:\host</HostFolder>",
                r"</MappedFolder>",
                r"<MappedFolder>",
                r"<HostFolder>C:\host2</HostFolder>",
                r"</MappedFolder>",
                r"</MappedFolders>",
                r"</Configuration>"
            ),
        );
    }

    #[test]
    fn logon_command_accepts_string_like_inputs() {
        let config = SandboxConfig::new()
            .logon_command("cmd.exe")
            .logon_command(OsStr::new("powershell.exe"));

        assert_eq!(
            config.to_os_string(),
            concat!(
                r"<Configuration>",
                r"<LogonCommand>",
                r"<Command>powershell.exe</Command>",
                r"</LogonCommand>",
                r"</Configuration>"
            ),
        );
    }

    #[test]
    fn pretty_serializes_nested_configuration() {
        let config = SandboxConfig::new()
            .vgpu(false)
            .mapped_folder(
                MappedFolder::new(r"C:\host")
                    .sandbox_folder(r"C:\sandbox")
                    .read_only(true),
            )
            .logon_command("cmd.exe");

        assert_eq!(
            config.to_pretty_os_string(),
            indoc! {r"
                <Configuration>
                  <vGPU>Disable</vGPU>
                  <MappedFolders>
                    <MappedFolder>
                      <HostFolder>C:\host</HostFolder>
                      <SandboxFolder>C:\sandbox</SandboxFolder>
                      <ReadOnly>true</ReadOnly>
                    </MappedFolder>
                  </MappedFolders>
                  <LogonCommand>
                    <Command>cmd.exe</Command>
                  </LogonCommand>
                </Configuration>
            "},
        );
    }
}
