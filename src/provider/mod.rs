pub mod error;
pub mod exoscale;

use crate::configuration::CloudConfiguration;
use async_trait::async_trait;
use error::CloudProviderError;


#[async_trait]
pub trait CloudProvider {
    async fn probe(&mut self) -> Result<(CloudConfiguration, CloudInstance, Option<CloudInstanceGroup>), CloudProviderError>;

    async fn get_metadata_userdata(&self) -> Result<String, CloudProviderError>;
    async fn get_metadata_cloud_identifier(&self) -> Result<String, CloudProviderError>;
    async fn get_metadata_zone(&self) -> Result<String, CloudProviderError>;
    async fn get_metadata_instance_id(&self) -> Result<String, CloudProviderError>;
    async fn get_metadata_hostname(&self) -> Result<String, CloudProviderError>;

    async fn get_instance(&self, id: &str, zone: &str)
        -> Result<CloudInstance, CloudProviderError>;

    async fn get_instance_group(
        &self,
        id: &str,
        zone: &str,
    ) -> Result<CloudInstanceGroup, CloudProviderError>;
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CloudInstance {
    pub instance_id: String,
    pub manager_id: Option<String>,
    pub hostname: String,
    pub zone: String,
    pub ipv4_address: Option<String>,
    pub ipv6_address: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CloudInstanceGroup {
    pub instance_group_id: String,
    pub instances: Vec<CloudInstance>,
    pub size: usize,
}
