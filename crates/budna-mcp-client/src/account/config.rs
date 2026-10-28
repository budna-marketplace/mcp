use std::time::Duration;

use reqwest::{Client, Url, redirect::Policy};

use super::{AccountApiClient, Credential, valid_credential};
use crate::{ClientBuildError, ClientConfig};

impl AccountApiClient {
    pub fn new(
        config: ClientConfig,
        issuer: &str,
        resource: &str,
        service_secret: String,
    ) -> Result<Self, ClientBuildError> {
        let issuer = Url::parse(issuer).map_err(|_| ClientBuildError::AccountConfiguration)?;
        let resource_url =
            Url::parse(resource).map_err(|_| ClientBuildError::AccountConfiguration)?;
        let service_credential = Credential::new(service_secret);
        if issuer.origin() != config.base_url().origin()
            || issuer.path() != "/"
            || issuer.query().is_some()
            || issuer.fragment().is_some()
            || !issuer.username().is_empty()
            || issuer.password().is_some()
            || resource_url.scheme() != "https"
            || resource_url.host_str().is_none()
            || resource_url.path() != "/account/mcp"
            || resource_url.query().is_some()
            || resource_url.fragment().is_some()
            || !resource_url.username().is_empty()
            || resource_url.password().is_some()
            || !(32..=512).contains(&service_credential.0.len())
            || !valid_credential(service_credential.0.as_str())
        {
            return Err(ClientBuildError::AccountConfiguration);
        }
        let http = Client::builder()
            .no_proxy()
            .connect_timeout(config.request_timeout().min(Duration::from_secs(10)))
            .timeout(config.request_timeout())
            .redirect(Policy::none())
            .user_agent(concat!("budna-mcp/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(ClientBuildError::HttpClient)?;
        Ok(Self {
            http,
            config,
            resource: resource.to_owned(),
            service_credential,
        })
    }
}
