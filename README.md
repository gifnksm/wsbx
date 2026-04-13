<!-- cargo-sync-rdme title [[ -->
# wsbx
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge [[ -->
[![Maintenance: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg?style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-badges-section)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/wsbx.svg?style=flat-square)](#license)
[![crates.io](https://img.shields.io/crates/v/wsbx.svg?logo=rust&style=flat-square)](https://crates.io/crates/wsbx)
[![docs.rs](https://img.shields.io/docsrs/wsbx.svg?logo=docs.rs&style=flat-square)](https://docs.rs/wsbx)
[![Rust: ^1.87.0](https://img.shields.io/badge/rust-^1.87.0-93450a.svg?logo=rust&style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![GitHub Actions: CI](https://img.shields.io/github/actions/workflow/status/gifnksm/wsbx/ci.yml.svg?label=CI&logo=github&style=flat-square)](https://github.com/gifnksm/wsbx/actions/workflows/ci.yml)
[![Codecov](https://img.shields.io/codecov/c/github/gifnksm/wsbx.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/gifnksm/wsbx)
<!-- cargo-sync-rdme ]] -->

<!-- cargo-sync-rdme rustdoc [[ -->
A type-safe Rust API for controlling Windows Sandbox through the `wsb` CLI.

This crate is supported on Windows only.

This crate provides:

* [`SandboxEnvironment`](https://docs.rs/wsbx/0.1.0/wsbx/environment/struct.SandboxEnvironment.html) for interacting with running sandbox instances
* [`SandboxEnvironmentBuilder`](https://docs.rs/wsbx/0.1.0/wsbx/environment/struct.SandboxEnvironmentBuilder.html) for starting new sandbox instances
* [`SandboxConfig`](https://docs.rs/wsbx/0.1.0/wsbx/config/struct.SandboxConfig.html) for constructing `.wsb`-compatible configuration XML
* [`SandboxId`](https://docs.rs/wsbx/0.1.0/wsbx/types/struct.SandboxId.html) for working with sandbox identifiers returned by `wsb`

The configuration model in [`config`](https://docs.rs/wsbx/0.1.0/wsbx/config/index.html) follows the Windows Sandbox `.wsb`
configuration format documented by Microsoft. For details, see
[Use and configure Windows Sandbox](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file).

The runtime API in this crate wraps the Windows Sandbox CLI. For command
behavior and arguments, see
[Windows Sandbox command line interface](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-cli).

## Examples

Start a sandbox with a typed configuration:

````rust,no_run
use wsbx::{SandboxConfig, SandboxEnvironment};

let environment = SandboxEnvironment::builder()
    .config(
        SandboxConfig::new()
            .networking(false)
            .memory_in_mb(4096),
    )
    .start()?;
````

Attach to an existing sandbox by ID and execute a command:

````rust,no_run
use wsbx::{RunAs, SandboxEnvironment, SandboxId};

let id: SandboxId = "12345678-1234-1234-1234-1234567890ab".parse()?;
let environment = SandboxEnvironment::from_id(id);
let result = environment.exec("cmd.exe", RunAs::System)?;
let _exit_code = result.exit_code();
````

## Usage

Add this to your `Cargo.toml`:

````toml
[dependencies]
wsbx = "0.1.0"
````
<!-- cargo-sync-rdme ]] -->

## Minimum supported Rust version (MSRV)

The minimum supported Rust version is **Rust 1.87.0**.
At least the last 3 versions of stable Rust are supported at any given time.

While a crate is a pre-release status (0.x.x) it may have its MSRV bumped in a patch release.
Once a crate has reached 1.x, any MSRV bump will be accompanied by a new minor version.

## License

This project is licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
