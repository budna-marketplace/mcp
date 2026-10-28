# Account Read profile

Budna MCP is intended to grow into a full integration with the Budna API,
including the actions available in the app. Account access begins with the
read-only tools below. Future actions require their own permissions and
appropriate safeguards, including confirmation for sensitive operations.
Customers control which capabilities they grant to a connected application
and can revoke its access. This direction does not make future actions
available in the current profile.

Account Read is a separate authenticated Streamable HTTP resource at
`/account/mcp`. Public Explore remains the default profile and its public
client never carries account credentials. Account Read is unavailable over
stdio; the embedded Marketplace Explorer remains public.

## Deployment configuration

Set `BUDNA_MCP_PROFILE=account-read` and
`BUDNA_MCP_TRANSPORT=streamable-http`, an explicit
`BUDNA_MCP_ACCOUNT_ISSUER` origin and an explicit HTTPS
`BUDNA_MCP_ACCOUNT_RESOURCE` ending in `/account/mcp`. The configured API must
have the same origin as the issuer. Configure exact HTTP Host/Origin
allowlists for the deployment. The listener remains on loopback behind the
operator's HTTPS reverse proxy.

Provision `BUDNA_MCP_SERVICE_SECRET` separately through the deployment's secret
store. It is a dedicated service credential, never a customer password,
marketplace session or signing key. No command-line secret option is provided.
The account profile fails startup when required configuration is missing.

The operator must register supported OAuth clients and their exact HTTPS
redirects with the authorization service. Discovery is published at
`/.well-known/oauth-protected-resource/account/mcp`; the root well-known path
also returns the resource metadata. The authorization service must advertise
its authorization-code/PKCE S256 flow and support the customer's browser
sign-in, explicit consent, token rotation and per-application revocation.

## Permissions and tools

| Tool | Scope | Output |
| --- | --- | --- |
| `get_my_account` | `account:read` | Account ID, personal/company kind and optional public account label |
| `list_my_listings` | `listings:read` | Bounded non-draft listing summaries with ID, title, status, type and update time |

The listing tool accepts only `page` (1–10000) and `limit` (1–50). The account
is always derived from the verified grant; no account, seller or company ID
can be supplied as authority. The first contract supports personal and company
owner accounts. Member delegation requires a separate reviewed capability.
These scopes do not authorize drafts, private prices, precise locations,
contact details, third-party identities, security settings or mutations.

All account and marketplace text is untrusted data. Output schemas and explicit
projection structs are enforced independently of backend response fields.
Additional response fields are ignored. Timestamps are Unix milliseconds.

## Credential boundary

Every request presents an MCP-audience bearer token in `Authorization`.
Credentials in cookies, query strings, tool arguments or MCP session IDs do
not authenticate an account. The transport uses stateless requests and checks
each request independently. Invalid credentials receive HTTP 401 with OAuth
discovery information; missing tool permissions receive HTTP 403 and a scope
challenge.

The resource exchanges an inbound token at the configured authorization
service using the dedicated service credential. The request is JSON
`{"subject_token":"<MCP token>","resource":"<exact resource>"}` to
`mcp/oauth/exchange` beneath the configured API base. The response contains a
separate bearer API token, expiry, scope, subject, grant ID, client ID and
resource. Tokens are never returned as tool output or embedded in resources.

Only that short-lived delegated API token is used for `GET mcp/account` and
`GET mcp/listings` beneath the API base. The API must independently enforce
grant revocation, expiry, scope, account standing and ownership, and must reject
these credentials on all other capabilities. The MCP token is never forwarded
to a marketplace resource endpoint. The connector stores no refresh tokens or
customer sessions.

## Operations and activation

Authorization and account responses must bypass proxy/CDN caching and stale
serving. Protect the origin, enforce request/rate limits and provision service
credentials separately per deployment. Logs contain bounded operation outcomes
and durations, never tokens, account identifiers, request text or raw payloads.
The credential exchange and reads have timeouts, concurrency and byte limits.

Before activation, validate supported host/client protocol versions, browser
connection and revocation, cross-account isolation, credential rotation and
failure recovery using synthetic accounts. Complete privacy review of the
account data delivered to each AI host. Disconnecting blocks future access;
it does not recall information already delivered to another service. This
source profile does not establish a live deployment or compliance approval.
