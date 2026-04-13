//! Types for interacting with running Windows Sandbox environments through the
//! `wsb` CLI.
//!
//! This module provides the main runtime API for starting sandbox instances,
//! attaching to existing ones by ID, and invoking operations such as command
//! execution, folder sharing, remote connection, and IP address lookup.
//!
//! For the underlying CLI behavior and arguments, see
//! [Windows Sandbox command line interface](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).

use std::{ffi::OsString, net::Ipv4Addr, path::PathBuf};

use crate::{
    RunAs, SandboxConfig, SandboxError, SandboxId,
    cli::{self, ExecResponse, IpResponse, ListResponse, StartResponse},
};

/// A builder for starting a new Windows Sandbox environment.
///
/// Use this builder to optionally specify a sandbox ID and startup
/// configuration before calling [`start`](Self::start).
///
/// This builder ultimately invokes the Windows Sandbox CLI. For command
/// behavior and arguments, see
/// [Windows Sandbox command line interface](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).
///
/// # Examples
///
/// Start a sandbox with a typed configuration:
///
/// ```rust,no_run
/// use wsbx::{SandboxConfig, SandboxEnvironment};
///
/// let environment = SandboxEnvironment::builder()
///     .config(SandboxConfig::new().networking(false))
///     .start()?;
/// # Ok::<(), wsbx::SandboxError>(())
/// ```
///
/// Start a sandbox with raw configuration XML:
///
/// ```rust,no_run
/// use wsbx::SandboxEnvironment;
///
/// let environment = SandboxEnvironment::builder()
///     .raw_config(r"<Configuration><Networking>Disable</Networking></Configuration>")
///     .start()?;
/// # Ok::<(), wsbx::SandboxError>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct SandboxEnvironmentBuilder {
    id: Option<SandboxId>,
    config: Option<OsString>,
}

impl SandboxEnvironmentBuilder {
    /// Creates a new builder with no explicit ID or configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the sandbox ID to use when starting the environment.
    ///
    /// If omitted, `wsb` chooses the ID.
    #[must_use]
    pub fn id<I>(mut self, id: I) -> Self
    where
        I: Into<SandboxId>,
    {
        self.id = Some(id.into());
        self
    }

    /// Sets the startup configuration using a typed [`SandboxConfig`].
    #[must_use]
    pub fn config<C>(mut self, config: C) -> Self
    where
        C: Into<SandboxConfig>,
    {
        self.config = Some(config.into().to_os_string());
        self
    }

    /// Sets the startup configuration using raw XML text.
    ///
    /// This bypasses [`SandboxConfig`] and passes the provided XML directly to
    /// the `wsb start --config` option.
    #[must_use]
    pub fn raw_config<S>(mut self, config: S) -> Self
    where
        S: Into<OsString>,
    {
        self.config = Some(config.into());
        self
    }

    /// Starts a new Windows Sandbox environment.
    ///
    /// On success, returns a [`SandboxEnvironment`] handle for the started
    /// sandbox.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb start` command fails or if its JSON response
    /// can't be deserialized.
    pub fn start(self) -> Result<SandboxEnvironment, SandboxError> {
        let Self { id, config } = self;
        let StartResponse { id } = cli::start(id, config)?;
        Ok(SandboxEnvironment { id })
    }
}

/// A handle to a Windows Sandbox environment.
///
/// This type can represent either a sandbox that was started through
/// [`SandboxEnvironmentBuilder::start`] or an existing sandbox referenced by
/// its [`SandboxId`].
///
/// The operations on this type wrap the Windows Sandbox CLI. For command
/// behavior and arguments, see
/// [Windows Sandbox command line interface](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).
#[derive(Debug, Clone)]
pub struct SandboxEnvironment {
    id: SandboxId,
}

impl SandboxEnvironment {
    /// Creates a builder for starting a new sandbox environment.
    #[must_use]
    pub fn builder() -> SandboxEnvironmentBuilder {
        SandboxEnvironmentBuilder::new()
    }

    /// Lists the currently running sandbox environments for the current user.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb list` command fails or if its JSON response
    /// can't be deserialized.
    pub fn list() -> Result<Vec<SandboxEnvironment>, SandboxError> {
        let ListResponse {
            windows_sandbox_environments: environments,
        } = cli::list()?;
        Ok(environments
            .into_iter()
            .map(|env| SandboxEnvironment { id: env.id })
            .collect())
    }

    /// Creates a handle for an existing sandbox environment by ID.
    ///
    /// This does not start a new sandbox.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wsbx::{SandboxEnvironment, SandboxId};
    ///
    /// let id: SandboxId = "12345678-1234-1234-1234-1234567890ab".parse()?;
    /// let environment = SandboxEnvironment::from_id(id);
    /// assert_eq!(environment.id(), id);
    /// # Ok::<(), wsbx::SandboxError>(())
    /// ```
    #[must_use]
    pub fn from_id(id: SandboxId) -> Self {
        Self { id }
    }

    /// Returns the sandbox ID associated with this environment.
    #[must_use]
    pub fn id(&self) -> SandboxId {
        self.id
    }

    /// Executes a command in the sandbox.
    ///
    /// The command is executed using the specified [`RunAs`] context.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb exec` command fails or if its JSON response
    /// can't be deserialized.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wsbx::{RunAs, SandboxEnvironment, SandboxId};
    ///
    /// let id: SandboxId = "12345678-1234-1234-1234-1234567890ab".parse()?;
    /// let environment = SandboxEnvironment::from_id(id);
    /// let result = environment.exec("cmd.exe", RunAs::System)?;
    /// let _exit_code = result.exit_code();
    /// # Ok::<(), wsbx::SandboxError>(())
    /// ```
    pub fn exec<C, R>(&self, command: C, run_as: R) -> Result<ExecResult, SandboxError>
    where
        C: Into<OsString>,
        R: Into<RunAs>,
    {
        self.exec_impl(command.into(), run_as.into(), None)
    }

    /// Executes a command in the sandbox with an explicit working directory.
    ///
    /// The command is executed using the specified [`RunAs`] context and starts
    /// in `working_directory`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb exec` command fails or if its JSON response
    /// can't be deserialized.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use wsbx::{RunAs, SandboxEnvironment, SandboxId};
    ///
    /// let id: SandboxId = "12345678-1234-1234-1234-1234567890ab".parse()?;
    /// let environment = SandboxEnvironment::from_id(id);
    /// let result = environment.exec_in("cmd.exe", RunAs::System, r"C:\Windows")?;
    /// let _exit_code = result.exit_code();
    /// # Ok::<(), wsbx::SandboxError>(())
    /// ```
    pub fn exec_in<C, R, P>(
        &self,
        command: C,
        run_as: R,
        working_directory: P,
    ) -> Result<ExecResult, SandboxError>
    where
        C: Into<OsString>,
        R: Into<RunAs>,
        P: Into<PathBuf>,
    {
        self.exec_impl(
            command.into(),
            run_as.into(),
            Some(working_directory.into()),
        )
    }

    fn exec_impl(
        &self,
        command: OsString,
        run_as: RunAs,
        working_directory: Option<PathBuf>,
    ) -> Result<ExecResult, SandboxError> {
        let ExecResponse { exit_code } = cli::exec(self.id, command, run_as, working_directory)?;
        Ok(ExecResult { exit_code })
    }

    /// Shares a host folder with the sandbox.
    ///
    /// If `allow_write` is `true`, the sandbox is allowed to modify the shared
    /// folder on the host.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb share` command fails.
    pub fn share<P, Q>(
        &self,
        host_path: P,
        sandbox_path: Q,
        allow_write: bool,
    ) -> Result<(), SandboxError>
    where
        P: Into<PathBuf>,
        Q: Into<PathBuf>,
    {
        cli::share(self.id, host_path.into(), sandbox_path.into(), allow_write)
    }

    /// Stops the sandbox environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb stop` command fails.
    pub fn stop(&self) -> Result<(), SandboxError> {
        cli::stop(self.id)
    }

    /// Opens a remote session to the sandbox environment.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb connect` command fails.
    pub fn connect(&self) -> Result<(), SandboxError> {
        cli::connect(self.id)
    }

    /// Returns the network information reported by the sandbox.
    ///
    /// # Errors
    ///
    /// Returns an error if the `wsb ip` command fails or if its JSON response
    /// can't be deserialized.
    pub fn ip(&self) -> Result<IpResult, SandboxError> {
        let IpResponse { networks } = cli::ip(self.id)?;
        Ok(IpResult {
            networks: networks
                .into_iter()
                .map(|network| IpNetwork {
                    ipv4_address: network.ipv4_address,
                })
                .collect(),
        })
    }
}

/// The result of executing a command in a sandbox.
#[derive(Debug)]
pub struct ExecResult {
    exit_code: i32,
}

impl ExecResult {
    /// Returns the process exit code reported by `wsb exec`.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

/// Network information returned by [`SandboxEnvironment::ip`].
#[derive(Debug)]
pub struct IpResult {
    networks: Vec<IpNetwork>,
}

impl IpResult {
    /// Returns the networks reported for the sandbox.
    #[must_use]
    pub fn networks(&self) -> &[IpNetwork] {
        &self.networks
    }
}

/// A single network entry returned by [`SandboxEnvironment::ip`].
#[derive(Debug)]
pub struct IpNetwork {
    ipv4_address: Ipv4Addr,
}

impl IpNetwork {
    /// Returns the IPv4 address for this network entry.
    #[must_use]
    pub fn ipv4_address(&self) -> Ipv4Addr {
        self.ipv4_address
    }
}
