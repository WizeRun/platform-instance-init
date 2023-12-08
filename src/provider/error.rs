use crate::http_client::HttpError;

#[derive(Clone, Debug, PartialEq)]
pub enum CloudProviderError {
    AuthenticationError,
    NotAvailable,
    ResourceUnreachable,
    ConfigurationError,
}

impl From<HttpError> for CloudProviderError {
    fn from(_: HttpError) -> Self {
        CloudProviderError::ResourceUnreachable
    }
}
