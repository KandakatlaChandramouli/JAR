mod cli;
mod error;
mod process;
mod sandbox;

use cli::parse_args;
use sandbox::{Sandbox, SandboxConfig};
use std::env;
use std::process;

fn main() {
    match run() {
        Ok(exit_code) => {
            process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("[jar] error: {}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<i32, error::JarError> {
    let command = parse_args(env::args())?;

    match command {
        cli::Command::Run { executable, args } => {
            let config = SandboxConfig { executable, args };
            let sandbox = Sandbox::new(config);
            sandbox.run()
        }
    }
}
