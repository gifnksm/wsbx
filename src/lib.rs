//! A type-safe Rust API for controlling Windows Sandbox through the `wsb` CLI.
//!
//! This crate is supported on Windows only.
//!
//! This crate provides:
//!
//! - [`SandboxEnvironment`] for interacting with running sandbox instances
//! - [`SandboxEnvironmentBuilder`] for starting new sandbox instances
//! - [`SandboxConfig`] for constructing `.wsb`-compatible configuration XML
//! - [`SandboxId`] for working with sandbox identifiers returned by `wsb`
//!
//! The configuration model in [`config`] follows the Windows Sandbox `.wsb`
//! configuration format documented by Microsoft. For details, see
//! [Use and configure Windows Sandbox](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file).
//!
//! The runtime API in this crate wraps the Windows Sandbox CLI. For command
//! behavior and arguments, see
//! [Windows Sandbox command line interface](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).
//!
//! # Examples
//!
//! Start a sandbox with a typed configuration:
//!
//! ```rust,no_run
//! use wsbx::{SandboxConfig, SandboxEnvironment};
//!
//! let environment = SandboxEnvironment::builder()
//!     .config(
//!         SandboxConfig::new()
//!             .networking(false)
//!             .memory_in_mb(4096),
//!     )
//!     .start()?;
//! # Ok::<(), wsbx::SandboxError>(())
//! ```
//!
//! Attach to an existing sandbox by ID and execute a command:
//!
//! ```rust,no_run
//! use wsbx::{RunAs, SandboxEnvironment, SandboxId};
//!
//! let id: SandboxId = "12345678-1234-1234-1234-1234567890ab".parse()?;
//! let environment = SandboxEnvironment::from_id(id);
//! let result = environment.exec("cmd.exe", RunAs::System)?;
//! let _exit_code = result.exit_code();
//! # Ok::<(), wsbx::SandboxError>(())
//! ```
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! wsbx = "0.1.0"
//! ```

#![doc(html_root_url = "https://docs.rs/wsbx/0.1.0")]
#![warn(clippy::pedantic, missing_docs)]

#[cfg(not(windows))]
compile_error!("wsbx is supported on Windows only and requires Windows Sandbox and the `wsb` CLI.");

pub use crate::{
    config::SandboxConfig,
    environment::{SandboxEnvironment, SandboxEnvironmentBuilder},
    error::SandboxError,
    types::{RunAs, SandboxId},
};

mod cli;
pub mod config;
pub mod environment;
mod error;
mod types;
mod xml;
