use crate::error::JarError;

#[derive(Debug, PartialEq)]
pub enum Command {
    Run {
        executable: String,
        args: Vec<String>,
    },
}

pub fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Command, JarError> {
    // Skip binary name
    args.next();

    let cmd = args.next().ok_or_else(|| {
        JarError::InvalidArgs("Missing command. Usage: jar run <program> [args...]".to_string())
    })?;

    if cmd != "run" {
        return Err(JarError::InvalidArgs(format!("Unknown command: {}", cmd)));
    }

    let executable = args.next().ok_or_else(|| {
        JarError::InvalidArgs("Missing executable. Usage: jar run <program> [args...]".to_string())
    })?;

    let cmd_args: Vec<String> = args.collect();

    Ok(Command::Run {
        executable,
        args: cmd_args,
    })
}
