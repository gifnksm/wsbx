use std::{ffi::OsString, path::PathBuf};

use clap::Parser as _;
use color_eyre::eyre::{self, WrapErr as _};
use wsbx::{RunAs, SandboxEnvironment, SandboxId};

#[derive(Debug, clap::Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(alias = "StartSandbox")]
    Start {
        #[clap(long)]
        id: Option<SandboxId>,
        #[clap(short, long)]
        config: Option<OsString>,
    },
    #[clap(alias = "ListRunningSandboxes")]
    List,
    #[clap(alias = "Execute")]
    Exec {
        #[clap(long)]
        id: SandboxId,
        #[clap(short, long)]
        command: OsString,
        #[clap(short, long)]
        run_as: RunAsArg,
        #[clap(short = 'd', long)]
        working_directory: Option<PathBuf>,
    },
    #[clap(alias = "ShareFolder")]
    Share {
        #[clap(long)]
        id: SandboxId,
        #[clap(short = 'f', long)]
        host_path: PathBuf,
        #[clap(short, long)]
        sandbox_path: PathBuf,
        #[clap(short, long)]
        allow_write: bool,
    },
    #[clap(alias = "StopSandbox")]
    Stop {
        #[clap(long)]
        id: SandboxId,
    },
    #[clap(alias = "ConnectToSandbox")]
    Connect {
        #[clap(long)]
        id: SandboxId,
    },
    #[clap(alias = "GetIpAddress")]
    Ip {
        #[clap(long)]
        id: SandboxId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "PascalCase")]
pub enum RunAsArg {
    ExistingLogin,
    System,
}

impl From<RunAsArg> for RunAs {
    fn from(value: RunAsArg) -> Self {
        match value {
            RunAsArg::ExistingLogin => Self::ExistingLogin,
            RunAsArg::System => Self::System,
        }
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.command {
        Command::Start { id, config } => {
            let mut builder = SandboxEnvironment::builder();
            if let Some(id) = id {
                builder = builder.id(id);
            }
            if let Some(config) = config {
                builder = builder.raw_config(config);
            }
            let env = builder
                .start()
                .wrap_err("failed to start windows sandbox environment")?;
            eprintln!("sandbox started successfully");
            eprintln!("  Sandbox ID: {}", env.id());
        }
        Command::List => {
            let environments = SandboxEnvironment::list()
                .wrap_err("failed to list windows sandbox environments")?;
            eprintln!("sandbox environments:");
            for env in environments {
                eprintln!("  {}", env.id());
            }
        }
        Command::Exec {
            id,
            command,
            run_as,
            working_directory,
        } => {
            let environment = SandboxEnvironment::from_id(id);
            let res = if let Some(working_directory) = working_directory {
                environment.exec_in(command, run_as, working_directory)
            } else {
                environment.exec(command, run_as)
            }
            .wrap_err("failed to exec command in windows sandbox environment")?;
            eprintln!("process exited with exit code {}", res.exit_code());
        }
        Command::Share {
            id,
            host_path,
            sandbox_path,
            allow_write,
        } => {
            let environment = SandboxEnvironment::from_id(id);
            environment
                .share(host_path, sandbox_path, allow_write)
                .wrap_err("failed to share folder with windows sandbox environment")?;
        }
        Command::Stop { id } => {
            let environment = SandboxEnvironment::from_id(id);
            environment
                .stop()
                .wrap_err("failed to stop windows sandbox environment")?;
        }
        Command::Connect { id } => {
            let environment = SandboxEnvironment::from_id(id);
            environment
                .connect()
                .wrap_err("failed to connect to windows sandbox environment")?;
        }
        Command::Ip { id } => {
            let environment = SandboxEnvironment::from_id(id);
            let res = environment
                .ip()
                .wrap_err("failed to get IP address of the windows sandbox environment")?;
            eprintln!("IP addresses:");
            for network in res.networks() {
                eprintln!("  {}", network.ipv4_address());
            }
        }
    }

    Ok(())
}
