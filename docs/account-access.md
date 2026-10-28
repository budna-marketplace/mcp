# Account Read profile

Budna MCP is intended to grow into a full integration with the Budna API,
including the actions available in the app. Account access begins with the
read-only tools below. Future actions require their own permissions and
appropriate safeguards, including confirmation for sensitive operations.
Customers control which capabilities they grant to a connected application
and can revoke its access. This direction does not make future actions
available in the current profile.

Account Read is a separate authenticated Streamable HTTP resource at
`/account/mcp`. Public Explore remains the default profile: its tools need no
Budna account, sign-in or API key, and its public client never carries account
credentials. Connecting an account is optional and only unlocks account tools.
Account Read is unavailable over
stdio; the embedded Marketplace Explorer remains public.

Applications hosting the resource can use `budna_mcp_server::account_http_router`.
It includes authentication, discovery, request limits, Host/Origin checks and
CORS. The host owns TLS, listener binding and graceful shutdown through the
supplied cancellation token. The standalone application's listener stays on
loopback.

Hosts offering both profiles can merge the account router with
`budna_mcp_server::stateless_public_http_router`, which serves anonymous tools
at `/mcp` without sessions or replica affinity. Both builders retain request-body,
Host, Origin and CORS controls; account authentication stays on `/account/mcp`.

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
| `list_my_listings` | `listings:read` | Bounded own listing summaries, including drafts, with ID, nullable title, status, type and update time |
| `get_my_listing` | `listings:read` | Owner-visible details for one own listing, including drafts and the owner's reserve price |

The listing collection accepts only `page` (1–10000) and `limit` (1–50). Detail
accepts only a canonical `listing_id`. The account
is always derived from the verified grant; no account, seller or company ID
can be supplied as authority. Reads cover listings whose recorded seller is
the logged-in personal or company principal account. They do not switch company
workspaces or use member permissions; member delegation needs a separate
reviewed capability.
The same listing permission covers own drafts and owner-visible prices. A
missing or foreign listing is unavailable; public visibility never substitutes
for ownership. Drafts retain null titles and descriptions when unfinished.

Detail includes title, description, category, condition, type, currency, market,
status, quantity, image IDs and optional ThumbHashes, shipping options and a
coarse location reference with city/region/country, attributes and their display labels, tags, timestamps,
views and placement flags. Auction details include prices, owner reserve,
round outcome, bid count, increment and next minimum bid. Fixed-price and live
round terms retain their exact amounts and timing. Safe showcase and live
session metadata provides context without playback credentials. Attributes with
complex values expose their app display text and a `json_display_only` marker
rather than arbitrary JSON; unsupported complex shapes have a null display value.
Buyer protection fee amounts carry the listing's currency.

The projection omits precise addresses, contacts, other people's identities,
internal row IDs, moderation or risk notes, staff metadata, tag claim codes,
credentials and raw response data. Free-form listing content can contain
information the owner entered; granting access lets the chosen AI host receive
that content, including unpublished drafts and reserve prices. These scopes do
not authorize security settings or mutations.

All account and marketplace text is untrusted data. Output schemas and explicit
projection structs are enforced independently of backend response fields.
Additional response fields are ignored. Timestamps are Unix milliseconds. Money is
`{"amount":"<exact decimal>","currency_code":"<ISO code>"}`. Detail bounds images
and tags to 50 each, attributes to 100, title text to 4096 bytes and description
text to 40000 bytes, with a shared encoded-text budget. `truncated` reports when
these output limits shorten a detail response.

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
`GET mcp/listings` and `GET mcp/listings/{listing_id}` beneath the API base. The API must independently enforce
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
