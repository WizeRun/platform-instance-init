mod api;
mod configuration;

use crate::configuration::CloudConfiguration;
use crate::http_client::HttpClient;
use crate::provider::error::CloudProviderError;
use crate::provider::{CloudInstance, CloudInstanceGroup, CloudProvider};
use async_trait::async_trait;
use hyper::header::AUTHORIZATION;
use log::{info, debug, error};
use std::collections::HashMap;

pub use api::{build_signature, ExoscaleInstance, ExoscaleInstanceManager, ExoscaleInstancePool};
pub use configuration::ExoscaleCloudProviderConfiguration;

const EXOSCALE_CLOUD_IDENTIFIER: &str = "Exoscale Compute Platform";

const EXOSCALE_METADATA_DEFAULT_TIMEOUT_SECS: u64 = 5;
const EXOSCALE_API_DEFAULT_TIMEOUT_SECS: u64 = 5;

pub struct ExoscaleCloudProvider {
    credentials: Option<ExoscaleAPICredentials>,
    metadata_client: HttpClient,
    api_client: HttpClient,
}

#[derive(Clone, Debug)]
pub struct ExoscaleAPICredentials {
    pub api_key: String,
    pub api_secret: String,
}

impl ExoscaleCloudProvider {
    pub fn new() -> ExoscaleCloudProvider {
        let metadata_client = HttpClient::new(EXOSCALE_METADATA_DEFAULT_TIMEOUT_SECS);
        let api_client = HttpClient::new(EXOSCALE_API_DEFAULT_TIMEOUT_SECS);

        ExoscaleCloudProvider {
            credentials: None,
            metadata_client,
            api_client,
        }
    }

    pub fn set_api_timeout(&mut self, timeout_secs: u64) {
        self.api_client.set_timeout(timeout_secs);
    }

    pub fn set_api_credentials(&mut self, credentials: ExoscaleAPICredentials) {
        self.credentials = Some(credentials);
    }

    async fn metadata_get(&self, path: &str) -> Result<String, CloudProviderError> {
        debug!("Retrieving metadata from path: {}", path);
        let uri = format!("http://169.254.169.254/latest/{}", path);
        Ok(self.metadata_client.request_get(uri).await?)
    }

    async fn api_get<T>(&self, zone: &str, path: &str) -> Result<T, CloudProviderError>
    where
        T: serde::de::DeserializeOwned,
    {
        debug!("Retrieving API data from path: {}", path);
        let path = format!("/v2/{}", path);
        let uri = format!("https://api-{}.exoscale.com{}", zone, path);

        let credentials = self
            .credentials
            .clone()
            .ok_or(CloudProviderError::AuthenticationError)?;

        let signature = build_signature(credentials, "GET", path, None, None).await?;

        let mut headers = HashMap::new();
        headers.insert(AUTHORIZATION.to_string(), signature);

        let response = self
            .api_client
            .request_get_with_headers(uri, headers)
            .await?;

        serde_json::from_str(&response).map_err(|err| {
            error!("Exoscale API deserialization: {}", err);
            CloudProviderError::NotAvailable
        })
    }

    pub async fn probe_basic_instance_data(&self) -> Result<CloudInstance, CloudProviderError> {
        let instance_id = self.get_metadata_instance_id().await?;
        debug!("Found instance id = {}", instance_id);
    
        let hostname = self.get_metadata_hostname().await?;
        debug!("Found hostname = {}", hostname);
    
        let zone = self.get_metadata_zone().await?;
        debug!("Found zone = {}", zone);
    
        Ok(CloudInstance {
            instance_id,
            hostname,
            zone,
            ..CloudInstance::default()
        })
    }

    pub async fn probe_advanced_instance_data(&mut self, configuration: &CloudConfiguration, instance: &CloudInstance) -> Result<(CloudInstance, Option<CloudInstanceGroup>), CloudProviderError> {
        info!("Configuring Exoscale API client");
        let api_options = configuration.provider.exoscale.clone().ok_or_else(|| {
            info!("Unable to get Exoscale provider configuration");
            CloudProviderError::ConfigurationError
        })?;

        self.set_api_credentials(ExoscaleAPICredentials {
            api_key: api_options.api_key,
            api_secret: api_options.api_secret,
        });
        self.set_api_timeout(api_options.api_timeout_secs);

        info!("Loading instance data from API");

        let instance = self
            .get_instance(instance.instance_id.as_str(), instance.zone.as_str())
            .await?;

        if let Some(manager_id) = instance.manager_id.clone() {
            let mut pool;
            info!("Waiting for all instances to be ready");
            loop {
                pool = self
                    .get_instance_group(manager_id.as_str(), instance.zone.as_str())
                    .await?;

                match pool.instances.len() == pool.size {
                    true => break,
                    false => {
                        debug!("Not yet fully available");
                        tokio::time::sleep(std::time::Duration::from_secs(
                            api_options.api_retry_delay_secs,
                        ))
                        .await;
                    }
                }
            }

            Ok((instance, Some(pool)))
        } else {
            Ok((instance, None))
        }
    }
}

#[async_trait]
impl CloudProvider for ExoscaleCloudProvider {
    async fn probe(&mut self) -> Result<(CloudConfiguration, CloudInstance, Option<CloudInstanceGroup>), CloudProviderError> {
        debug!("Probing Exoscale cloud provider");
        if self.get_metadata_cloud_identifier().await? != EXOSCALE_CLOUD_IDENTIFIER {
            debug!("Not running in Exoscale cloud");
            return Err(CloudProviderError::NotAvailable);
        }

        info!("Loading configuration from user_data");
        let user_data = self.get_metadata_userdata().await?;
        let configuration = CloudConfiguration::from_str(user_data.as_str()).ok_or_else(|| {
            error!("Unable to parse cloud configuration");
            CloudProviderError::ConfigurationError
        })?;

        info!("Loading instance data from cloud metadata server");
        let instance = self.probe_basic_instance_data().await?;

        let (instance, instance_group) = match self.probe_advanced_instance_data(&configuration, &instance).await {
            Ok(instance_data) => instance_data,
            _ => (instance, None)
        };

        Ok((configuration, instance, instance_group))
    }

    async fn get_metadata_userdata(&self) -> Result<String, CloudProviderError> {
        Ok(self.metadata_get("user-data").await?)
    }

    async fn get_metadata_cloud_identifier(&self) -> Result<String, CloudProviderError> {
        Ok(self.metadata_get("meta-data/cloud-identifier").await?)
    }

    async fn get_metadata_zone(&self) -> Result<String, CloudProviderError> {
        Ok(self.metadata_get("meta-data/availability-zone").await?)
    }

    async fn get_metadata_instance_id(&self) -> Result<String, CloudProviderError> {
        Ok(self.metadata_get("meta-data/instance-id").await?)
    }

    async fn get_metadata_hostname(&self) -> Result<String, CloudProviderError> {
        Ok(self.metadata_get("meta-data/local-hostname").await?)
    }

    async fn get_instance(
        &self,
        id: &str,
        zone: &str,
    ) -> Result<CloudInstance, CloudProviderError> {
        let path = format!("/instance-pool/{}", id);
        let instance: ExoscaleInstance = self.api_get(zone, &path).await?;

        Ok(CloudInstance {
            instance_id: instance.id,
            manager_id: instance.manager.map(|manager| manager.id),
            hostname: instance.name,
            zone: zone.to_string(),
            ipv4_address: instance.ipv4_address,
            ipv6_address: instance.ipv6_address,
        })
    }

    async fn get_instance_group(
        &self,
        id: &str,
        zone: &str,
    ) -> Result<CloudInstanceGroup, CloudProviderError> {
        let path = format!("/instance-pool/{}", id);
        let instance_pool: ExoscaleInstancePool = self.api_get(zone, &path).await?;

        let mut instances = Vec::new();
        for instance in instance_pool.instances {
            instances.push(
                self.get_instance(instance.id.as_str(), zone.as_ref())
                    .await?,
            );
        }

        Ok(CloudInstanceGroup {
            instance_group_id: String::from(id),
            instances,
            size: instance_pool.size,
        })
    }
}
