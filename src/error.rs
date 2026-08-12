use std::fmt;

#[derive(Debug)]
pub enum JarError {
    InvalidArgs(String),
    Validation(String),
    Execution(String),
    Io(std::io::Error),
    Syscall(nix::Error),
}

impl fmt::Display for JarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JarError::InvalidArgs(msg) => write!(f, "Argument error: {}", msg),
            JarError::Validation(msg) => write!(f, "Validation error: {}", msg),
            JarError::Execution(msg) => write!(f, "Execution error: {}", msg),
            JarError::Io(err) => write!(f, "IO error: {}", err),
            JarError::Syscall(err) => write!(f, "Syscall error: {}", err),
        }
    }
}

impl From<std::io::Error> for JarError {
    fn from(err: std::io::Error) -> Self {
        JarError::Io(err)
    }
}

impl From<nix::Error> for JarError {
    fn from(err: nix::Error) -> Self {
        JarError::Syscall(err)
    }
}
