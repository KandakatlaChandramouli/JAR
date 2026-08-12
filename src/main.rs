mod capabilities;
mod cgroup;
mod cli;
mod error;
mod image;
mod overlay;
mod process;
mod sandbox;
mod seccomp;

use cli::{parse_args, print_help, print_version, Command};
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
        Command::Help => {
            print_help();
            Ok(0)
        }
        Command::Version => {
            print_version();
            Ok(0)
        }
        Command::Run(opts) => {
            let config = SandboxConfig::from(opts);
            let sandbox = Sandbox::new(config);
            sandbox.run()
        }
    }
}
