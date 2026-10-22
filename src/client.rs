use std::num::NonZeroU32;

use rustopus::{
    reqwest::{self, Client, RequestBuilder},
    Arm, Quota, StatusPolicy,
};
use serde_json::Value;

use crate::{connection_limiter::ConnectionLimitLayer, error::APIError};

/// OANDA REST environment. Practice is the default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Environment {
    #[default]
    Practice,
    Live,
}

impl Environment {
    pub fn rest_url(self) -> &'static str {
        match self {
            Self::Practice => "https://api-fxpractice.oanda.com",
            Self::Live => "https://api-fxtrade.oanda.com",
        }
    }
}

/// REST client settings. Defaults to practice, 8 concurrent requests,
/// 100 requests/second, 5 retries, and environment-based proxy discovery.
#[derive(Clone)]
pub struct ClientConfig {
    pub environment: Environment,
    pub concurrency_limit: usize,
    pub rate_limit: usize,
    pub retry_attempts: usize,
    /// An explicit HTTP/HTTPS proxy URL, overriding environment proxy settings.
    /// `None` preserves reqwest's environment-based proxy discovery.
    pub proxy: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Practice,
            concurrency_limit: 8,
            rate_limit: 100,
            retry_attempts: 5,
            proxy: None,
        }
    }
}

#[derive(Clone)]
pub struct OandaClient {
    http: Client,
    arm: Arm<StatusPolicy>,
    account_id: Option<String>,
    api_key: String,
    base_url: String,
}

impl OandaClient {
    /// Create a practice REST client. `buffer_size` is retained for source
    /// compatibility; rustopus waits in the calling future rather than a queue.
    /// `rate_limit` must be 1..=100 and `concurrency_limit` must be positive.
    /// `retry_attempts` counts retries after the initial attempt.
    pub fn new(
        account_id: Option<&str>,
        api_key: &str,
        buffer_size: usize,
        concurrency_limit: usize,
        rate_limit: usize,
        retry_attempts: usize,
    ) -> Result<Self, APIError> {
        Self::new_with_environment(
            account_id,
            api_key,
            buffer_size,
            concurrency_limit,
            rate_limit,
            retry_attempts,
            Environment::Practice,
        )
    }

    /// Create a REST client for an explicitly selected environment.
    pub fn new_with_environment(
        account_id: Option<&str>,
        api_key: &str,
        _buffer_size: usize,
        concurrency_limit: usize,
        rate_limit: usize,
        retry_attempts: usize,
        environment: Environment,
    ) -> Result<Self, APIError> {
        Self::new_with_config(
            account_id,
            api_key,
            ClientConfig {
                environment,
                concurrency_limit,
                rate_limit,
                retry_attempts,
                proxy: None,
            },
        )
    }

    /// Create a REST client with explicit settings, including an optional proxy.
    /// Proxy configuration applies to all requests and is shared across clones.
    pub fn new_with_config(
        account_id: Option<&str>,
        api_key: &str,
        config: ClientConfig,
    ) -> Result<Self, APIError> {
        let ClientConfig {
            environment,
            concurrency_limit,
            rate_limit,
            retry_attempts,
            proxy,
        } = config;
        if !(1..=100).contains(&rate_limit) || concurrency_limit == 0 {
            return Err(APIError::Other(
                "rate_limit must be 1..=100 and concurrency_limit must be positive".into(),
            ));
        }
        let attempts = retry_attempts
            .checked_add(1)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| APIError::Other("retry_attempts is too large".into()))?;
        let mut http = Client::builder()
            .http1_only()
            .pool_idle_timeout(None)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connector_layer(ConnectionLimitLayer);
        if let Some(proxy) = proxy {
            http = http.proxy(reqwest::Proxy::all(proxy)?);
        }
        let http = http.build()?;
        let arm = Arm::builder()
            .name("oanda-rest")
            .rate(
                Quota::per_second(NonZeroU32::new(rate_limit as u32).unwrap())
                    .allow_burst(NonZeroU32::new(1).unwrap()),
            )
            .suckers(concurrency_limit)
            .max_attempts(attempts)
            .build(http.clone());
        Ok(Self {
            http,
            arm,
            account_id: account_id.map(str::to_owned),
            api_key: api_key.to_owned(),
            base_url: environment.rest_url().to_owned(),
        })
    }

    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }

    pub fn get_account_id(&self) -> Option<&String> {
        self.account_id.as_ref()
    }

    async fn send_request(&self, request: RequestBuilder) -> Result<Value, APIError> {
        let request = request.bearer_auth(&self.api_key).build()?;
        let response = self.arm.send(request).await?;
        let value = serde_json::from_slice(&response.body)?;
        Self::check_response(Ok(value)).await
    }

    pub async fn get(&mut self, url: &str) -> Result<Value, APIError> {
        let request = self.http.get(format!("{}{}", self.base_url, url));
        self.send_request(request).await
    }

    pub async fn patch(&mut self, url: &str, body: &Value) -> Result<Value, APIError> {
        let request = self
            .http
            .patch(format!("{}{}", self.base_url, url))
            .json(body);
        self.send_request(request).await
    }

    pub async fn check_response(response: Result<Value, APIError>) -> Result<Value, APIError> {
        let value = response?;
        if let Some(message) = value.get("errorMessage").and_then(Value::as_str) {
            Err(APIError::Other(message.to_owned()))
        } else {
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests;
