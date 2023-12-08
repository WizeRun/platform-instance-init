use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, Default)]
pub struct ExoscaleCloudProviderConfiguration {
    pub api_key: String,
    pub api_secret: String,
    #[serde(default = "default_api_timeout_secs")]
    pub api_timeout_secs: u64,
    #[serde(default = "default_api_retry_delay_secs")]
    pub api_retry_delay_secs: u64,
}

fn default_api_timeout_secs() -> u64 {
    5
}

fn default_api_retry_delay_secs() -> u64 {
    5
}
