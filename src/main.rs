mod configuration;
mod host;
mod http_client;
mod provider;

use crate::configuration::CloudConfiguration;
use crate::provider::error::CloudProviderError;
use crate::provider::exoscale::ExoscaleCloudProvider;
use crate::provider::{CloudInstance, CloudProvider, CloudInstanceGroup};
use env_logger::Env;
use log::info;

async fn probe_exoscale(
) -> Result<(CloudConfiguration, CloudInstance, Option<CloudInstanceGroup>), CloudProviderError> {
    let mut provider = ExoscaleCloudProvider::new();
    Ok(provider.probe().await?)
}

async fn cloud_init(configuration: CloudConfiguration, instance: CloudInstance) -> Result<(), host::HostError> {
    if host::set_instance_hostname(instance.hostname.clone()).is_ok() {
        info!("Hostname set to {}", instance.hostname);
    }

    host::ensure_directory("/var/lib/ssh".to_string())?;

    if host::ensure_ssh_hostkey("ed25519").is_ok() {
        info!("SSH ed25519 host key generated");
    }

    for (username, user_configuration) in configuration.host.user.into_iter() {
        if !user_configuration.ssh.authorized_keys.is_empty() {
            info!("Setting ssh keys for {}", username);
            let home = host::user_home(username);
            let keys = user_configuration.ssh.authorized_keys.join("\n");

            host::ensure_directory(format!("{}/.ssh", home))?;
            host::ensure_file(format!("{}/.ssh/authorized_keys", home), keys)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let (configuration, instance, _group) =
        if let Ok((configuration, instance, group)) = probe_exoscale().await {
            info!("Loaded cloud init data from Exoscale platform");
            (configuration, instance, group)
        } else {
            return;
        };
    
    if let Err(error) = cloud_init(configuration, instance).await {
        info!("Error: {:?}", error);
    }
}
