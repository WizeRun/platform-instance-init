use crate::provider::error::CloudProviderError;
use crate::provider::exoscale::ExoscaleAPICredentials;
use hmac::{Hmac, Mac, NewMac};
use log::error;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// A small subset of the API responses
// REF: https://openapi-v2.exoscale.com

#[derive(Clone, Deserialize, Debug)]
pub struct ExoscaleInstanceManager {
    #[serde(rename = "type")]
    pub manager_type: String,
    pub id: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ExoscaleInstance {
    pub id: String,
    pub name: String,
    pub manager: Option<ExoscaleInstanceManager>,
    #[serde(rename = "public-ip")]
    pub ipv4_address: Option<String>,
    #[serde(rename = "ipv6-address")]
    pub ipv6_address: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ExoscaleInstancePool {
    pub size: usize,
    pub instances: Vec<ExoscaleInstance>,
}

fn build_expiration_timestamp(timeout: u64) -> Result<u64, CloudProviderError> {
    Ok((SystemTime::now() + Duration::from_secs(timeout))
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            error!("Error while building expiration timestamp: {:?}", err);
            CloudProviderError::AuthenticationError
        })?
        .as_secs())
}

fn build_authentication_header(
    api_key: &str,
    param_args: &str,
    expiration: u64,
    signature: String,
) -> String {
    if !param_args.is_empty() {
        format!(
            "EXO2-HMAC-SHA256 credential={},signed-query-args={},expires={},signature={}",
            api_key, param_args, expiration, signature
        )
    } else {
        format!(
            "EXO2-HMAC-SHA256 credential={},expires={},signature={}",
            api_key, expiration, signature
        )
    }
}

pub async fn build_signature(
    credentials: ExoscaleAPICredentials,
    method: &str,
    path: String,
    body: Option<&str>,
    params: Option<HashMap<&str, &str>>,
) -> Result<String, CloudProviderError> {
    let expiration = build_expiration_timestamp(120)?;

    let body = body.unwrap_or_default();
    let params = params.unwrap_or_default();

    let (param_args, param_values) = params.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut args, mut values), (key, value)| {
            args.push(key);
            values.push(value);
            (args, values)
        },
    );

    let param_args = param_args.join(":");
    let param_values = param_values.join(":");

    let message = format!(
        "{} {}\n{}\n{}\n\n{}",
        method, path, body, param_values, expiration
    );

    let mut mac =
        Hmac::<Sha256>::new_from_slice(credentials.api_secret.as_bytes()).map_err(|err| {
            error!("Error while building signature: {:?}", err);
            CloudProviderError::AuthenticationError
        })?;
    mac.update(message.as_bytes());

    let signature = mac.finalize().into_bytes();
    let signature = base64::encode(signature);

    let auth_header = build_authentication_header(
        credentials.api_key.as_str(),
        param_args.as_str(),
        expiration,
        signature,
    );

    Ok(auth_header)
}
