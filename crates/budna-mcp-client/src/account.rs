mod config;
mod models;

use models::ExchangeResponse;

use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{
    Client, RequestBuilder, Url,
    header::{ACCEPT, AUTHORIZATION, HeaderValue},
};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use zeroize::Zeroizing;

pub use models::{AccountListingPage, AccountListingSummary, AccountSummary};

use crate::{ClientConfig, ClientError, is_listing_id, is_user_id, models::ApiEnvelope};

pub const ACCOUNT_READ_SCOPE: &str = "account:read";
pub const LISTINGS_READ_SCOPE: &str = "listings:read";
const MAX_ACCOUNT_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_TOKEN_RESPONSE_BYTES: usize = 16 * 1024;

#[derive(Clone)]
struct Credential(Arc<Zeroizing<String>>);

impl Credential {
    fn new(value: String) -> Self {
        Self(Arc::new(Zeroizing::new(value)))
    }

    fn header(&self, operation: &'static str) -> Result<HeaderValue, ClientError> {
        let mut value = HeaderValue::from_str(&format!("Bearer {}", self.0.as_str()))
            .map_err(|_| invalid_response(operation))?;
        value.set_sensitive(true);
        Ok(value)
    }
}

impl fmt::Debug for Credential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted]")
    }
}

impl<'de> Deserialize<'de> for Credential {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::new)
    }
}

/// A request-scoped credential returned by the configured authorization service.
#[derive(Clone)]
pub struct DelegatedAccess {
    credential: Credential,
    subject: String,
    scopes: Arc<[String]>,
    expires: Instant,
}

impl fmt::Debug for DelegatedAccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DelegatedAccess([redacted])")
    }
}

impl DelegatedAccess {
    pub fn allows(&self, scope: &str) -> bool {
        Instant::now() < self.expires && self.scopes.iter().any(|granted| granted == scope)
    }
}

/// Account requests use explicit delegated credentials, never public-client defaults.
#[derive(Clone)]
pub struct AccountApiClient {
    http: Client,
    config: ClientConfig,
    resource: String,
    service_credential: Credential,
}

impl AccountApiClient {
    pub async fn exchange(&self, token: &str) -> Result<DelegatedAccess, ClientError> {
        const OPERATION: &str = "authorize_account_request";
        if token.len() > 512 || !valid_credential(token) {
            return Err(unauthorized(OPERATION));
        }
        let endpoint = self.endpoint(OPERATION, "mcp/oauth/exchange")?;
        let request = self
            .http
            .post(endpoint)
            .header(AUTHORIZATION, self.service_credential.header(OPERATION)?)
            .json(&serde_json::json!({"subject_token": token, "resource": self.resource}));
        let result: ExchangeResponse = self
            .send(OPERATION, request, MAX_TOKEN_RESPONSE_BYTES)
            .await?;
        let scopes = result
            .scope
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if result.token_type != "Bearer"
            || !(1..=60).contains(&result.expires_in)
            || result.resource != self.resource
            || !is_user_id(&result.subject)
            || !valid_label(&result.grant_id)
            || !valid_label(&result.client_id)
            || !valid_credential(result.access_token.0.as_str())
            || result.access_token.0.len() > 512
            || scopes.is_empty()
            || scopes.len() > 2
            || scopes
                .iter()
                .any(|scope| !matches!(scope.as_str(), ACCOUNT_READ_SCOPE | LISTINGS_READ_SCOPE))
            || (scopes.len() == 2 && scopes[0] == scopes[1])
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(DelegatedAccess {
            credential: result.access_token,
            subject: result.subject,
            scopes: Arc::from(scopes),
            expires: Instant::now() + Duration::from_secs(result.expires_in),
        })
    }

    pub async fn account(&self, access: &DelegatedAccess) -> Result<AccountSummary, ClientError> {
        const OPERATION: &str = "get_my_account";
        self.require_scope(access, ACCOUNT_READ_SCOPE, OPERATION)?;
        let result: AccountSummary = self.get(OPERATION, "mcp/account", access, &[]).await?;
        if result.id != access.subject
            || !matches!(result.account_type.as_str(), "personal" | "company")
            || result
                .display_name
                .as_ref()
                .is_some_and(|name| name.len() > 4096)
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(result)
    }

    pub async fn listings(
        &self,
        access: &DelegatedAccess,
        page: u32,
        limit: u32,
    ) -> Result<AccountListingPage, ClientError> {
        const OPERATION: &str = "list_my_listings";
        self.require_scope(access, LISTINGS_READ_SCOPE, OPERATION)?;
        if !(1..=10_000).contains(&page) || !(1..=50).contains(&limit) {
            return Err(ClientError::InvalidInput {
                operation: OPERATION,
                code: "INVALID_INPUT",
                message: "page must be 1..10000 and limit 1..50",
            });
        }
        let result: AccountListingPage = self
            .get(
                OPERATION,
                "mcp/listings",
                access,
                &[("page", page), ("limit", limit)],
            )
            .await?;
        let pagination = &result.pagination;
        if pagination.page != i64::from(page)
            || pagination.limit != i64::from(limit)
            || pagination.total < 0
            || pagination.total_pages
                != pagination.total / i64::from(limit)
                    + i64::from(pagination.total % i64::from(limit) != 0)
            || result.items.len() > limit as usize
            || result.items.len() as i64
                > pagination
                    .total
                    .saturating_sub(i64::from(page - 1) * i64::from(limit))
                    .max(0)
            || result.items.iter().any(|listing| {
                !is_listing_id(&listing.id)
                    || listing.title.len() > 4096
                    || listing.updated_at < 0
                    || !matches!(
                        listing.status.as_str(),
                        "active" | "sold" | "expired" | "cancelled"
                    )
                    || !matches!(
                        listing.listing_type.as_str(),
                        "auction" | "fixed_price" | "auction_fixed_price" | "in_stock"
                    )
            })
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(result)
    }

    fn require_scope(
        &self,
        access: &DelegatedAccess,
        scope: &str,
        operation: &'static str,
    ) -> Result<(), ClientError> {
        if access.allows(scope) {
            Ok(())
        } else {
            Err(ClientError::Api {
                operation,
                status: 403,
                code: None,
                retry_after: None,
            })
        }
    }

    fn endpoint(&self, operation: &'static str, path: &str) -> Result<Url, ClientError> {
        self.config
            .endpoint(path)
            .map_err(|source| ClientError::Endpoint { operation, source })
    }

    async fn get<T: DeserializeOwned>(
        &self,
        operation: &'static str,
        path: &str,
        access: &DelegatedAccess,
        query: &[(&str, u32)],
    ) -> Result<T, ClientError> {
        let request = self
            .http
            .get(self.endpoint(operation, path)?)
            .header(AUTHORIZATION, access.credential.header(operation)?)
            .query(query);
        let envelope: ApiEnvelope<T> = self
            .send(operation, request, MAX_ACCOUNT_RESPONSE_BYTES)
            .await?;
        if !envelope.success {
            return Err(ClientError::UnsuccessfulEnvelope { operation });
        }
        envelope.data.ok_or(ClientError::MissingData { operation })
    }

    async fn send<T: DeserializeOwned>(
        &self,
        operation: &'static str,
        request: RequestBuilder,
        limit: usize,
    ) -> Result<T, ClientError> {
        let started = Instant::now();
        let result = async {
            let mut response = request
                .header(ACCEPT, "application/json")
                .send()
                .await
                .map_err(|source| crate::request_error(operation, source))?;
            let status = response.status();
            if !status.is_success() {
                return Err(ClientError::Api {
                    operation,
                    status: status.as_u16(),
                    code: None,
                    retry_after: None,
                });
            }
            if response
                .content_length()
                .is_some_and(|length| length > limit as u64)
            {
                return Err(ClientError::ResponseTooLarge {
                    operation,
                    limit_bytes: limit,
                });
            }
            let mut body = Zeroizing::new(Vec::new());
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|source| crate::request_error(operation, source))?
            {
                if chunk.len() > limit.saturating_sub(body.len()) {
                    return Err(ClientError::ResponseTooLarge {
                        operation,
                        limit_bytes: limit,
                    });
                }
                body.extend_from_slice(&chunk);
            }
            serde_json::from_slice(&body)
                .map_err(|source| ClientError::Decode { operation, source })
        }
        .await;
        tracing::debug!(
            operation,
            elapsed_ms = crate::elapsed_millis(started),
            outcome = if result.is_ok() { "success" } else { "failure" },
            "Budna delegated request completed"
        );
        result
    }
}

fn valid_credential(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/=".contains(&byte))
}

fn valid_label(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.bytes().all(|byte| byte.is_ascii_graphic())
}

fn invalid_response(operation: &'static str) -> ClientError {
    ClientError::InvalidResponse { operation }
}

fn unauthorized(operation: &'static str) -> ClientError {
    ClientError::Api {
        operation,
        status: 401,
        code: None,
        retry_after: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, header, method, path},
    };

    const RESOURCE: &str = "https://mcp.example.test/account/mcp";
    const SERVICE: &str = "synthetic-service-credential-for-local-tests";
    const USER_A: &str = "user_00000000-0000-4000-8000-000000000001";
    const USER_B: &str = "user_00000000-0000-4000-8000-000000000002";

    fn client(api: &MockServer) -> AccountApiClient {
        AccountApiClient::new(
            ClientConfig::new(api.uri()).unwrap_or_else(|error| panic!("mock URL: {error}")),
            &api.uri(),
            RESOURCE,
            SERVICE.to_owned(),
        )
        .unwrap_or_else(|error| panic!("mock client: {error}"))
    }

    async fn exchange(api: &MockServer, token: &str, api_token: &str, subject: &str, scope: &str) {
        Mock::given(method("POST")).and(path("/api/v1/mcp/oauth/exchange"))
            .and(header("authorization", format!("Bearer {SERVICE}")))
            .and(body_json(json!({"subject_token":token,"resource":RESOURCE})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token":api_token,"token_type":"Bearer","expires_in":60,"scope":scope,
                "subject":subject,"grant_id":"00000000-0000-4000-8000-000000000003","client_id":"synthetic-client","resource":RESOURCE,
                "extra":"ignored-backend-sentinel"
            }))).mount(api).await;
    }

    #[tokio::test]
    async fn concurrent_accounts_use_only_their_own_downstream_credentials() {
        let api = MockServer::start().await;
        exchange(
            &api,
            "incoming-a",
            "downstream-a",
            USER_A,
            ACCOUNT_READ_SCOPE,
        )
        .await;
        exchange(
            &api,
            "incoming-b",
            "downstream-b",
            USER_B,
            ACCOUNT_READ_SCOPE,
        )
        .await;
        for (credential, user) in [("downstream-a", USER_A), ("downstream-b", USER_B)] {
            Mock::given(method("GET")).and(path("/api/v1/mcp/account"))
                .and(header("authorization", format!("Bearer {credential}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{
                    "id":user,"account_type":"personal","display_name":null,"extra":"ignored-backend-sentinel"
                }}))).mount(&api).await;
        }
        let client = client(&api);
        let (a, b) = tokio::join!(client.exchange("incoming-a"), client.exchange("incoming-b"));
        let a = a.unwrap_or_else(|error| panic!("account A exchange: {error}"));
        let b = b.unwrap_or_else(|error| panic!("account B exchange: {error}"));
        assert!(!format!("{a:?}").contains("downstream-a"));
        let (a, b) = tokio::join!(client.account(&a), client.account(&b));
        assert_eq!(
            a.unwrap_or_else(|error| panic!("account A: {error}")).id,
            USER_A
        );
        assert_eq!(
            b.unwrap_or_else(|error| panic!("account B: {error}")).id,
            USER_B
        );
        for request in api.received_requests().await.unwrap_or_default() {
            assert!(request.headers.get("cookie").is_none());
            if request.method == "GET" {
                let auth = request
                    .headers
                    .get("authorization")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default();
                assert!(matches!(
                    auth,
                    "Bearer downstream-a" | "Bearer downstream-b"
                ));
            }
        }
    }

    #[tokio::test]
    async fn missing_scope_and_cross_account_responses_fail_closed() {
        let api = MockServer::start().await;
        exchange(
            &api,
            "incoming-a",
            "downstream-a",
            USER_A,
            LISTINGS_READ_SCOPE,
        )
        .await;
        let client = client(&api);
        let access = client
            .exchange("incoming-a")
            .await
            .unwrap_or_else(|error| panic!("exchange: {error}"));
        assert!(matches!(
            client.account(&access).await,
            Err(ClientError::Api { status: 403, .. })
        ));
        assert_eq!(api.received_requests().await.unwrap_or_default().len(), 1);
        exchange(
            &api,
            "incoming-b",
            "downstream-b",
            USER_B,
            ACCOUNT_READ_SCOPE,
        )
        .await;
        Mock::given(method("GET")).and(path("/api/v1/mcp/account"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"id":USER_A,"account_type":"personal","display_name":"User A"}})))
            .mount(&api).await;
        let access = client
            .exchange("incoming-b")
            .await
            .unwrap_or_else(|error| panic!("exchange: {error}"));
        assert!(matches!(
            client.account(&access).await,
            Err(ClientError::InvalidResponse { .. })
        ));
    }

    #[tokio::test]
    async fn wrong_resource_and_unknown_scopes_are_rejected() {
        for (resource, scope) in [
            ("https://other.example.test/account/mcp", ACCOUNT_READ_SCOPE),
            (RESOURCE, "account:write"),
        ] {
            let api = MockServer::start().await;
            Mock::given(method("POST")).and(path("/api/v1/mcp/oauth/exchange"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "access_token":"downstream","token_type":"Bearer","expires_in":60,"scope":scope,
                    "subject":USER_A,"grant_id":"synthetic-grant","client_id":"synthetic-client","resource":resource
                }))).mount(&api).await;
            assert!(matches!(
                client(&api).exchange("incoming").await,
                Err(ClientError::InvalidResponse { .. })
            ));
        }
    }

    #[tokio::test]
    async fn account_reads_reject_drafts_invalid_pagination_and_oversized_responses() {
        let api = MockServer::start().await;
        exchange(&api, "incoming", "downstream", USER_A, LISTINGS_READ_SCOPE).await;
        let client = client(&api);
        let access = client
            .exchange("incoming")
            .await
            .unwrap_or_else(|error| panic!("exchange: {error}"));
        let mock = Mock::given(method("GET")).and(path("/api/v1/mcp/listings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{
                "items":[{"id":"listing_00000000-0000-4000-8000-000000000007","title":"synthetic","status":"draft","listing_type":"auction","updated_at":1700000000000_i64}],
                "pagination":{"page":1,"limit":20,"total":1,"total_pages":1}
            }}))).mount_as_scoped(&api).await;
        assert!(matches!(
            client.listings(&access, 1, 20).await,
            Err(ClientError::InvalidResponse { .. })
        ));
        drop(mock);
        assert!(matches!(
            client.listings(&access, 0, 20).await,
            Err(ClientError::InvalidInput { .. })
        ));
        Mock::given(method("GET"))
            .and(path("/api/v1/mcp/listings"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![
                b'x';
                MAX_ACCOUNT_RESPONSE_BYTES
                    + 1
            ]))
            .mount(&api)
            .await;
        assert!(matches!(
            client.listings(&access, 1, 20).await,
            Err(ClientError::ResponseTooLarge { .. })
        ));
    }
}
