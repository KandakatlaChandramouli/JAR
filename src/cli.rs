use crate::cgroup::ResourceLimits;
use crate::error::JarError;

#[derive(Debug, PartialEq)]
pub struct RunOptions {
    pub executable: String,
    pub args: Vec<String>,
    pub rootfs: Option<String>,
    pub image: Option<String>,
    pub limits: ResourceLimits,
    pub enable_seccomp: bool,
    pub drop_capabilities: bool,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Run(RunOptions),
    Help,
    Version,
}

pub fn parse_args(args: impl Iterator<Item = String>) -> Result<Command, JarError> {
    let mut args_iter = args.skip(1);

    let cmd = match args_iter.next() {
        Some(arg) => arg,
        None => return Ok(Command::Help),
    };

    match cmd.as_str() {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "-V" | "--version" | "version" => Ok(Command::Version),
        "run" => parse_run_args(args_iter),
        unknown => Err(JarError::InvalidArgs(format!(
            "Unknown command: '{}'. Run 'jar --help' for usage.",
            unknown
        ))),
    }
}

fn parse_run_args(mut args_iter: impl Iterator<Item = String>) -> Result<Command, JarError> {
    let mut rootfs = None;
    let mut image = None;
    let mut memory_max_bytes = None;
    let mut pids_max = None;
    let mut enable_seccomp = true;
    let mut drop_capabilities = true;
    let mut positionals = Vec::new();

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "--rootfs" => {
                rootfs = Some(args_iter.next().ok_or_else(|| {
                    JarError::InvalidArgs("Missing path for --rootfs option".to_string())
                })?);
            }
            "--image" => {
                image = Some(args_iter.next().ok_or_else(|| {
                    JarError::InvalidArgs("Missing path for --image option".to_string())
                })?);
            }
            "--memory" => {
                let val = args_iter.next().ok_or_else(|| {
                    JarError::InvalidArgs("Missing value for --memory option".to_string())
                })?;
                let bytes = val.parse::<u64>().map_err(|_| {
                    JarError::InvalidArgs(format!("Invalid memory byte limit: {}", val))
                })?;
                memory_max_bytes = Some(bytes);
            }
            "--pids" => {
                let val = args_iter.next().ok_or_else(|| {
                    JarError::InvalidArgs("Missing value for --pids option".to_string())
                })?;
                let max = val.parse::<u64>().map_err(|_| {
                    JarError::InvalidArgs(format!("Invalid pids max limit: {}", val))
                })?;
                pids_max = Some(max);
            }
            "--no-seccomp" => {
                enable_seccomp = false;
            }
            "--no-caps-drop" => {
                drop_capabilities = false;
            }
            "--" => {
                positionals.extend(args_iter);
                break;
            }
            opt if opt.starts_with('-') => {
                return Err(JarError::InvalidArgs(format!("Unknown option: {}", opt)));
            }
            executable => {
                positionals.push(executable.to_string());
                positionals.extend(args_iter);
                break;
            }
        }
    }

    if positionals.is_empty() {
        return Err(JarError::InvalidArgs(
            "Missing executable path. Usage: jar run [OPTIONS] <program> [args...]".to_string(),
        ));
    }

    let executable = positionals.remove(0);
    let args = positionals;

    let limits = ResourceLimits {
        memory_max_bytes,
        cpu_max_quota_us: None,
        cpu_max_period_us: None,
        pids_max,
    };

    Ok(Command::Run(RunOptions {
        executable,
        args,
        rootfs,
        image,
        limits,
        enable_seccomp,
        drop_capabilities,
    }))
}

pub fn print_help() {
    println!(
        "JAR v1.0.0 - A Linux Process-Isolation Container Runtime in Pure Rust\n\n\
Usage:\n  \
jar run [OPTIONS] <executable> [args...]\n  \
jar [COMMAND]\n\n\
Commands:\n  \
run           Execute a command inside an isolated container sandbox\n  \
help, -h      Print help information\n  \
version, -V   Print version information\n\n\
Run Options:\n  \
--image <PATH>    Path to OCI/Docker tarball archive (.tar, .tar.gz, .tgz)\n  \
--rootfs <PATH>   Path to custom rootfs target directory\n  \
--memory <BYTES>  Maximum physical memory limit in bytes (e.g., 536870912)\n  \
--pids <MAX>      Maximum allowed process count (fork-bomb defense)\n  \
--no-seccomp      Disable default Seccomp BPF syscall restriction profile\n  \
--no-caps-drop    Retain standard POSIX capabilities (do not strip caps)"
    );
}

pub fn print_version() {
    println!("jar version 1.0.0");
}
