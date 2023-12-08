use std::io::Error;

#[derive(Debug)]
pub enum HostError {
    HostnameError,
    SSHSetupError,
    IOError(Error),
}

impl From<Error> for HostError {
    fn from(error: Error) -> Self {
        Self::IOError(error)
    }
}