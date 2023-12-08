use crate::provider::exoscale::ExoscaleCloudProviderConfiguration;
use log::error;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Deserialize, Debug, Default)]
pub struct CloudConfiguration {
    #[serde(default = "default_provider_configuration")]
    pub provider: ProviderConfiguration,

    #[serde(default = "default_host_configuration")]
    pub host: HostConfiguration,
}

impl CloudConfiguration {
    pub fn from_str(configuration: &str) -> Option<CloudConfiguration> {
        match toml::from_str(configuration) {
            Ok(configuration) => Some(configuration),
            Err(e) => {
                error!("Error parsing configuration file: {}", e);
                None
            }
        }
    }
}

pub fn default_provider_configuration() -> ProviderConfiguration {
    ProviderConfiguration::default()
}

pub fn default_host_configuration() -> HostConfiguration {
    HostConfiguration::default()
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct ProviderConfiguration {
    pub exoscale: Option<ExoscaleCloudProviderConfiguration>,
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct HostConfiguration {
    pub user: HashMap<String, UserConfiguration>
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct UserConfiguration {
    pub ssh: UserSSHConfiguration
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct UserSSHConfiguration {
    pub authorized_keys: Vec<String>
}
