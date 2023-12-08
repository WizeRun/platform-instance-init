use hyper::client::HttpConnector;
use hyper::header::{InvalidHeaderName, InvalidHeaderValue};
use hyper::http::{HeaderName, HeaderValue};
use hyper::{Body, Client, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use log::{debug, error};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum HttpError {
    Timeout,
    TlsError,
    RequestError,
    TransportError,
    ResponseError,
    ClientError(String),
    ServerError(String),
}

impl From<std::io::Error> for HttpError {
    fn from(_: std::io::Error) -> Self {
        HttpError::TransportError
    }
}

impl From<rustls::Error> for HttpError {
    fn from(_: rustls::Error) -> Self {
        HttpError::TlsError
    }
}

impl From<rustls::InvalidMessage> for HttpError {
    fn from(_: rustls::InvalidMessage) -> Self {
        HttpError::TlsError
    }
}

impl From<hyper::Error> for HttpError {
    fn from(_: hyper::Error) -> Self {
        HttpError::TransportError
    }
}

impl From<hyper::http::Error> for HttpError {
    fn from(_: hyper::http::Error) -> Self {
        HttpError::TransportError
    }
}

impl From<InvalidHeaderValue> for HttpError {
    fn from(_: InvalidHeaderValue) -> Self {
        HttpError::RequestError
    }
}

impl From<InvalidHeaderName> for HttpError {
    fn from(_: InvalidHeaderName) -> Self {
        HttpError::RequestError
    }
}

pub struct HttpClient {
    client: Client<HttpsConnector<HttpConnector>>,
    timeout: Duration,
}

impl HttpClient {
    pub fn new(timeout: u64) -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_all_versions()
            .build();
        let client = Client::builder().build::<_, Body>(https);

        Self {
            client,
            timeout: Duration::from_secs(timeout),
        }
    }

    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = Duration::from_secs(timeout);
    }

    fn set_headers(
        &self,
        req: &mut Request<Body>,
        headers: &HashMap<String, String>,
    ) -> Result<(), HttpError> {
        for (k, v) in headers {
            let header_name = HeaderName::from_str(k)?;
            let header_value = HeaderValue::from_str(v)?;
            req.headers_mut().insert(header_name, header_value);
        }

        for (name, value) in headers {
            req.headers_mut()
                .insert(
                    HeaderName::from_str(name.as_str())?,
                    HeaderValue::from_str(value.as_str())?,
                )
                .ok_or(HttpError::RequestError)?;
        }
        Ok(())
    }

    async fn parse_response(&self, req: Request<Body>) -> Result<String, HttpError> {
        debug!("HTTP req: {:?}", req);

        let res = self.client.request(req);
        let res = tokio::time::timeout(self.timeout, res)
            .await
            .map_err(|e| {
                error!("Timeout while executing request: {:?}", e);
                HttpError::Timeout
            })??;

        debug!("HTTP res: {:?}", res);

        let status = res.status();
        let body = res.into_body();
        let body = hyper::body::to_bytes(body).await?;
        let body = String::from_utf8(body.to_vec()).map_err(|_| HttpError::ResponseError)?;

        debug!("HTTP body: {:?}", body);

        if status.is_client_error() {
            return Err(HttpError::ClientError(body));
        } else if status.is_server_error() {
            return Err(HttpError::ServerError(body));
        }

        Ok(body)
    }

    pub async fn request_get(&self, uri: String) -> Result<String, HttpError> {
        self.request_get_with_headers(uri, HashMap::new()).await
    }

    pub async fn request_get_with_headers(
        &self,
        uri: String,
        headers: HashMap<String, String>,
    ) -> Result<String, HttpError> {
        debug!("HTTP GET {}", uri);
        let mut req = Request::get(uri).body(Body::empty())?;
        self.set_headers(&mut req, &headers)?;
        self.parse_response(req).await
    }
}
