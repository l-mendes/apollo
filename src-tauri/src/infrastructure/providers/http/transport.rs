use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::application::errors::{ApplicationError, ApplicationErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Value,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Value,
}

#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn post_json(&self, request: HttpRequest) -> Result<HttpResponse, ApplicationError>;
}

pub struct ReqwestHttpTransport {
    client: reqwest::Client,
}

impl ReqwestHttpTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .build()
                .expect("reqwest client should build"),
        }
    }
}

impl Default for ReqwestHttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpTransport for ReqwestHttpTransport {
    async fn post_json(&self, request: HttpRequest) -> Result<HttpResponse, ApplicationError> {
        let mut builder = self
            .client
            .post(&request.url)
            .timeout(request.timeout)
            .json(&request.body);

        for (header_name, header_value) in &request.headers {
            builder = builder.header(header_name, header_value);
        }

        let response = builder.send().await.map_err(map_reqwest_error)?;
        let status = response.status().as_u16();
        let body = response.json::<Value>().await.map_err(|error| {
            ApplicationError::new(
                ApplicationErrorKind::Unknown,
                format!("failed to decode HTTP response body: {error}"),
            )
        })?;

        Ok(HttpResponse { status, body })
    }
}

fn map_reqwest_error(error: reqwest::Error) -> ApplicationError {
    if error.is_timeout() {
        return ApplicationError::new(ApplicationErrorKind::Timeout, error.to_string());
    }

    ApplicationError::new(ApplicationErrorKind::Unavailable, error.to_string())
}
