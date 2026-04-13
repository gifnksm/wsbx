use std::{
    ffi::OsString,
    fmt::Write as _,
    io::Read,
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::Deserialize;

use crate::{RunAs, SandboxError, SandboxId};

pub(crate) fn wsb_command() -> Command {
    let mut wsb = Command::new("wsb");
    wsb.arg("--raw");
    wsb
}

fn stderr_error<S>(message: S) -> Option<Box<dyn std::error::Error + Send + Sync + 'static>>
where
    S: AsRef<str> + Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    if message.as_ref().is_empty() {
        return None;
    }
    Some(message.into())
}

pub(crate) fn execute_wsb<R>(wsb: &mut Command) -> Result<R, SandboxError>
where
    R: for<'a> Deserialize<'a>,
{
    let output = wsb.output().map_err(SandboxError::ExecuteWsbCommand)?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        return Err(SandboxError::WsbCommand {
            status: output.status,
            source: stderr_error(message),
        });
    }
    let response =
        serde_json::from_slice(&output.stdout).map_err(SandboxError::DeserializeResponse)?;
    Ok(response)
}

pub(crate) fn execute_wsb_no_response(wsb: &mut Command) -> Result<(), SandboxError> {
    let output = wsb.output().map_err(SandboxError::ExecuteWsbCommand)?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        return Err(SandboxError::WsbCommand {
            status: output.status,
            source: stderr_error(message),
        });
    }
    Ok(())
}

// `wsb connect` is special: `Command::output()` waits until the remote session
// window is closed, while the CLI itself returns immediately after opening it.
//
// To preserve that behavior, this path uses `spawn` + `wait` and reads stderr
// only after exit. This could hang if `wsb connect` filled the stderr pipe, but
// in practice it is expected to emit little or no stderr output.
pub(crate) fn execute_wsb_no_response_no_stdout(wsb: &mut Command) -> Result<(), SandboxError> {
    let mut child = wsb
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SandboxError::ExecuteWsbCommand)?;
    let status = child.wait().map_err(SandboxError::ExecuteWsbCommand)?;
    if !status.success() {
        let mut message = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            if let Err(err) = stderr.read_to_string(&mut message) {
                let mut read_error_message =
                    format!("failed to read stderr from wsb process: {err}");
                if !message.is_empty() {
                    let _ = write!(read_error_message, "\npartial stderr: {message}");
                }
                message = read_error_message;
            }
        }
        return Err(SandboxError::WsbCommand {
            status,
            source: stderr_error(message),
        });
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct StartResponse {
    pub(crate) id: SandboxId,
}

pub(crate) fn start(
    id: Option<SandboxId>,
    config: Option<OsString>,
) -> Result<StartResponse, SandboxError> {
    let mut wsb = wsb_command();
    wsb.arg("start");
    if let Some(id) = id {
        wsb.arg("--id").arg(id.to_string());
    }
    if let Some(config) = config {
        wsb.arg("--config").arg(config);
    }
    execute_wsb(&mut wsb)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ListResponse {
    pub(crate) windows_sandbox_environments: Vec<ListEnvironment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ListEnvironment {
    pub(crate) id: SandboxId,
}

pub(crate) fn list() -> Result<ListResponse, SandboxError> {
    let mut wsb = wsb_command();
    wsb.arg("list");
    execute_wsb(&mut wsb)
}

pub(crate) fn exec(
    id: SandboxId,
    command: OsString,
    run_as: RunAs,
    working_directory: Option<PathBuf>,
) -> Result<ExecResponse, SandboxError> {
    let run_as_str = match run_as {
        RunAs::ExistingLogin => "ExistingLogin",
        RunAs::System => "System",
    };

    let mut wsb = wsb_command();
    wsb.arg("exec");
    wsb.arg("--id").arg(id.to_string());
    wsb.arg("--command").arg(command);
    wsb.arg("--run-as").arg(run_as_str);
    if let Some(working_directory) = working_directory {
        wsb.arg("--working-directory").arg(working_directory);
    }
    execute_wsb(&mut wsb)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ExecResponse {
    pub(crate) exit_code: i32,
}

pub(crate) fn share(
    id: SandboxId,
    host_path: PathBuf,
    sandbox_path: PathBuf,
    allow_write: bool,
) -> Result<(), SandboxError> {
    let mut wsb = wsb_command();
    wsb.arg("share");
    wsb.arg("--id").arg(id.to_string());
    wsb.arg("--host-path").arg(host_path);
    wsb.arg("--sandbox-path").arg(sandbox_path);
    if allow_write {
        wsb.arg("--allow-write");
    }
    execute_wsb_no_response(&mut wsb)
}

pub(crate) fn stop(id: SandboxId) -> Result<(), SandboxError> {
    let mut wsb = wsb_command();
    wsb.arg("stop");
    wsb.arg("--id").arg(id.to_string());
    execute_wsb_no_response(&mut wsb)
}

pub(crate) fn connect(id: SandboxId) -> Result<(), SandboxError> {
    let mut wsb = wsb_command();
    wsb.arg("connect");
    wsb.arg("--id").arg(id.to_string());
    execute_wsb_no_response_no_stdout(&mut wsb)
}

pub(crate) fn ip(id: SandboxId) -> Result<IpResponse, SandboxError> {
    let mut wsb = wsb_command();
    wsb.arg("ip");
    wsb.arg("--id").arg(id.to_string());
    execute_wsb(&mut wsb)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct IpResponse {
    pub(crate) networks: Vec<IpNetwork>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct IpNetwork {
    #[serde(rename = "IpV4Address")]
    pub(crate) ipv4_address: Ipv4Addr,
}
