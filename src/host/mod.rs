pub use crate::host::error::HostError;
use log::error;
use std::fs;
use std::process::Command;

mod error;

pub fn set_instance_hostname(hostname: String) -> Result<(), HostError> {
    let mut cmd = Command::new("hostnamectl");
    cmd.arg("hostname").arg("--transient").arg(hostname);

    let output = cmd.output().map_err(|err| {
        error!("hostnamectl failed: {}", err);
        HostError::HostnameError
    })?;

    if !output.status.success() {
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            error!("hostnamectl failed: {}", stderr);
        }

        return Err(HostError::HostnameError);
    }

    Ok(())
}

pub fn ensure_ssh_hostkey(algorithm: &str) -> Result<(), HostError> {
    let mut cmd = Command::new("ssh-keygen");
    cmd
        .arg("-t").arg(algorithm)
        .arg("-f").arg(format!("/var/lib/ssh/ssh_host_{}_key", algorithm))
        .arg("-N").arg("");

    let output = cmd.output().map_err(|err| {
        error!("ssh-keygen failed: {}", err);
        HostError::SSHSetupError
    })?;

    if !output.status.success() {
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            error!("ssh-keygen failed: {}", stderr);
        }

        return Err(HostError::SSHSetupError);
    }

    Ok(())
}

pub fn user_home(username: String) -> String {
    // TODO: rely on linux std configuration instead!
    if username == "root" {
        "/root".to_string()
    } else {
        format!("/home/{}", username)
    }
}

pub fn ensure_directory(full_path: String) -> Result<(), HostError> {
    Ok(fs::create_dir_all(full_path)?)
}

pub fn ensure_file(full_path: String, content: String) -> Result<(), HostError> {
    Ok(fs::write(full_path, content)?)
}
