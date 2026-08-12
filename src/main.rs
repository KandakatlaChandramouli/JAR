mod cgroup;
mod cli;
mod error;
mod process;
mod sandbox;

use cli::parse_args;
use sandbox::{Sandbox, SandboxConfig};
use std::env;
use std::process::exit;

fn main() {
    match run() {
        Ok(exit_code) => {
            exit(exit_code);
        }
        Err(e) => {
            eprintln!("[jar] error: {}", e);
            exit(1);
        }
    }
}

fn run() -> Result<i32, error::JarError> {
    let command = parse_args(env::args())?;

    match command {
        cli::Command::Run { executable, args } => {
            let config = SandboxConfig {
                executable,
                args,
                rootfs: None,
                limits: None,
            };
            let sandbox = Sandbox::new(config);
            sandbox.run()
        }
    }
}
