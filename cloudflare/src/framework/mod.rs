/*!
This module controls how requests are sent to Cloudflare's API, and how responses are parsed from it.
 */
pub mod async_api;
pub mod auth;
// There is no blocking implementation for wasm.
#[cfg(all(feature = "blocking", not(target_arch = "wasm32")))]
pub mod blocking_api;
pub mod endpoint;
pub mod response;

use serde::Serialize;
use std::net::IpAddr;
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
/// Errors encountered while trying to connect to the Cloudflare API
pub enum Error {
    /// An error via the `reqwest` crate
    #[error("Reqwest returned an error when connecting to the Cloudflare API: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Serialize, Clone, Debug)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// Used as a parameter to API calls that search for a resource (e.g. DNS records).
/// Tells the API whether to return results that match all search requirements or at least one (any).
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SearchMatch {
    /// Match all search requirements
    All,
    /// Match at least one search requirement
    Any,
}

/// Which environment (host path) to use for API calls
#[derive(Debug)]
pub enum Environment {
    /// The production endpoint: `https://api.cloudflare.com/client/v4`
    Production,
    /// A custom endpoint
    Custom(url::Url),
}

impl<'a> From<&'a Environment> for url::Url {
    fn from(environment: &Environment) -> Self {
        match environment {
            Environment::Production => {
                url::Url::parse("https://api.cloudflare.com/client/v4/").unwrap()
            }
            Environment::Custom(url) => url.clone(),
        }
    }
}

// There is no blocking support for wasm.
#[cfg(all(feature = "blocking", not(target_arch = "wasm32")))]
/// Synchronous Cloudflare API client.
pub struct HttpApiClient {
    environment: Environment,
    credentials: auth::Credentials,
    http_client: reqwest::blocking::Client,
}

/// Configuration for the API client. Allows users to customize its behaviour.
pub struct HttpApiClientConfig {
    /// The maximum time limit for an API request. If a request takes longer than this, it will be
    /// cancelled.
    /// Note: this configuration has no effect when the target is wasm32.
    pub http_timeout: Duration,
    /// A default set of HTTP headers which will be sent with each API request.
    pub default_headers: http::HeaderMap,
    /// A specific IP to use when establishing a connection
    /// Note: this configuration has no effect when the target is wasm32.
    pub resolve_ip: Option<IpAddr>,
}

impl Default for HttpApiClientConfig {
    fn default() -> Self {
        HttpApiClientConfig {
            http_timeout: Duration::from_secs(30),
            default_headers: http::HeaderMap::default(),
            resolve_ip: None,
        }
    }
}
