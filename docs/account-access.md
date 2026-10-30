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

`AccountMcpServer::with_public` also serves every public Explore tool and the
Marketplace Explorer resources on `/account/mcp`, so a connected client reads
public and own data through one connection. Public tools need no delegated
permission and still use only the credential-free public client.

Hosts offering both profiles can merge the account router with
`budna_mcp_server::public_http_router`, which serves anonymous tools at `/mcp`
using stateless JSON responses without sessions or replica affinity.
`stateless_public_http_router` remains an equivalent alias. Both profiles retain
request-body, Host, Origin and CORS controls; account authentication stays on
`/account/mcp`.

## Connecting a client

Budna hosts the account resource at `https://api.budna.se/account/mcp`. Add it
to an MCP client as a remote Streamable HTTP server. The client discovers
Budna's authorization server, opens the browser to sign you in at
`https://connect.budna.se` and shows the permissions it requests; you choose
which to grant. You can reduce or remove an app's access at any time under
Account → Security → Connected apps.

Budna accepts these public clients, none of which needs a client secret:

| Client | Setup |
| --- | --- |
| Claude Code | `claude mcp add --transport http budna https://api.budna.se/account/mcp`, then authenticate with `/mcp` |
| Claude (custom connector) | Add the URL with OAuth client ID `budna-claude` |
| Codex | `codex mcp add budna --url https://api.budna.se/account/mcp`, then `codex mcp login budna` |
| ChatGPT (custom connector) | Add the URL with OAuth client ID `budna-chatgpt` and no client secret |
| Visual Studio Code | Add the URL as an HTTP MCP server |
| goose | Add the URL as a remote extension |
| Gemini CLI | Add the URL with `oauth.clientId` set to `budna-gemini-cli` |
| Cursor | Add the URL with OAuth client ID `budna-cursor` |

Claude Code, Codex, Visual Studio Code and goose identify themselves with their
published client metadata, which Budna recognizes without extra configuration.
Other clients are rejected: Budna offers no dynamic client registration.

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
| `get_my_profile` | `profile:read` | Own profile and status: username, display name, bio, city, rating, level and badges, verification, standing with outstanding debt, active subscription and onboarding steps; never birth date, addresses or payment preferences |
| `get_my_achievements` | `profile:read` | Achievements with name, description, category, progress, target and unlock time, plus the count of hidden ones still locked |
| `get_my_referral` | `profile:read` | Referral standing: whether the link is open, the code to share, counted and waiting referrals, leaderboard place, rewards and the next reward step; never who was referred |
| `get_my_legal_documents` | `profile:read` | Each legal document's current version, the version accepted and when, and whether acceptance is pending or blocking |
| `list_my_listings` | `listings:read` | Bounded own listing summaries, including drafts, with ID, nullable title, status, type and update time |
| `get_my_listing` | `listings:read` | Owner-visible details for one own listing, including drafts and the owner's reserve price |
| `list_my_bids` | `bids:read` | Own bids with listing ID and title, amount, placement time and status; optionally only currently winning bids |
| `list_my_offers` | `offers:read` | Offers sent or received with listing, role, amount, quantity, counter amount, status, last actor (`me` or `other_party`) and expiry |
| `list_my_watchlists` | `watchlist:read` | Own watchlists with ID, name, default flag, item count and timestamps |
| `list_my_watchlist_listings` | `watchlist:read` | Listing IDs and titles in one own watchlist |
| `list_my_saved_searches` | `watchlist:read` | Saved searches with name, search criteria and filters, alert frequency, active flag and match counts |
| `list_my_saved_search_matches` | `watchlist:read` | Public listings one own saved search matches now, with ID, title, shown price, type and end time |
| `list_my_live_reminders` | `watchlist:read` | Live streams the user asked to be reminded about, soonest first, with ID, title, status and scheduled start |
| `list_my_orders` | `orders:read` | Purchases or sales with listing, amount, quantity, order/payment/fulfillment status, carrier, tracking number, return status, deadlines and seller payout |
| `get_my_order` | `orders:read` | One own purchase or sale with tracking events, return, refund and dispute status and invoice numbers and totals, without documents |
| `list_my_disputes` | `orders:read` | Disputes about own purchases and sales with order, listing, side (`buyer` or `seller`), reason, status and timestamps |
| `get_my_dispute` | `orders:read` | One own dispute with both parties' statements (`me` or `other_party`, oldest first) and settlement offers with amount, status and deadline; never evidence files |
| `list_my_order_groups` | `orders:read` | Combined shipments with side, the other party's public username, status, member order IDs, item, buyer protection, fee, shipping and total amounts, delivery mode, carrier and deadlines |
| `get_my_order_group` | `orders:read` | One own combined shipment with the same fields; never addresses or service points |
| `list_my_auto_bids` | `bids:read` | Own automatic bids with listing, maximum, increment, current bid and status |
| `list_my_second_chance_offers` | `offers:read` | Second-chance offers received, with listing, price, your bid, status and window |
| `get_my_offer_history` | `offers:read` | One own offer's quantity and its negotiation steps (actor `me` or `other_party`, action, amount, time), oldest first |
| `get_my_money_summary` | `payments:read` | Per-currency totals of incoming proceeds (pending, on the way, paid out, on hold, in balance) and outgoing payments (paid, refunded) |
| `list_my_payment_activity` | `payments:read` | Newest-first payment activity with direction, kind, status, amount and related order or listing; continue with `cursor` |
| `list_my_payouts` | `payments:read` | Recent payouts with amount, status and timestamps |
| `list_my_debts` | `payments:read` | Debts to Budna with amount, status, reason and related order |
| `get_my_credits` | `payments:read` | Priority and boost credit balances per kind: granted, consumed, available and the next expiry |
| `get_my_payout_setup` | `payments:read` | Seller payment readiness, merchant kind, verification, payout, agreement and settlement account statuses and activated payment methods; never bank or account numbers |
| `list_my_conversations` | `messages:read` | Conversations with the other participant's public username, related listing or order, latest message preview and counts |
| `get_my_conversation` | `messages:read` | Messages in one own conversation: sender (`me` or `other_party`), text, type, related offer, image count, time and read state |
| `list_my_listing_chats` | `messages:read` | Public listing chats the user takes part in: listing, side, who wrote the latest message with their public username when it is someone else, its preview, unread count and last activity |
| `get_my_seller_insights` | `selling:read` | Engagement for `range` 7d, 30d (default) or 90d: status, live and period counts, a daily series, up to 10 top listings and the priority-credit balance |
| `list_my_boosts` | `selling:read` | Bought boosts with the boosted listing or collection, title, type, status, price, start and expiry |
| `get_my_listing_promotion` | `selling:read` | One own listing's current, queued and recent boosts, search priority with quoted price, video processing status without links, and a company listing's VAT treatment with the company default |
| `get_my_shipping_settings` | `selling:read` | Default carriers and the combined-shipping choice; never addresses |
| `list_my_showcases` | `selling:read` | Own collections including drafts: title, kind, status, dates, public item count and boost flag |
| `list_my_live_sessions` | `selling:read` | Own live streams: status, title, times, peak viewers, reminders, items, items sold and revenue |
| `list_my_listing_tags` | `selling:read` | In-store listing tags with listing, total and 7-day scans, last scan day and saves; never the printed tag code |
| `list_my_following` | `following:read` | Accounts the user follows, by public username and follow time |
| `list_my_followers` | `following:read` | Accounts following the user, by public username and follow time |
| `list_my_companies` | `company:read` | Company memberships with role and permissions, plus pending invitations addressed to the user |
| `get_my_company` | `company:read` | One own company: details, members by public username and role for managers, invoicing status for the company account |
| `list_my_notifications` | `notifications:read` | Notifications with type, priority, read state, time and related listing, order, offer or saved search |
| `get_my_notification_preferences` | `notifications:read` | Push, email and in-app switches, per-category switches and quiet hours |
| `list_my_ratings` | `ratings:read` | Ratings received or given with listing, 1–5 star value, comment and time |

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

Activity tools accept `page` (1–10000) and `limit` (1–50) plus only their
documented selectors: `winning_only` for bids, `role` (`sent`/`received`) and
`open_only` for offers, `watchlist_id` for watchlist listings,
`saved_search_id` for saved-search matches, `role` (`buyer`/`seller`) for
orders, `unread_only` for notifications and `direction` (`received`/`given`)
for ratings. The order, dispute, combined-shipment, offer-history and company
reads take only a positive
`order_id`, `dispute_id`, `group_id`, `offer_id` or `company_id`; a record that
does not exist or is not the account's own is unavailable. They read only
records where the connected account is the bidder, buyer, seller, watcher,
searcher, recipient, rater, ratee, a party to the dispute or a participant in
the conversation or chat. Orders, offers, ratings, combined shipments and chats
name the other party only by the public username the app shows; bidders on a
listing and referred people are never identified. They omit addresses, service
points, contact details, offer and order messages, notification text and
senders, dispute evidence, payment instruments, bank and account numbers,
printed tag codes and internal staff or risk data. Status values are backend
codes in lowercase snake case and may gain new values. Seller-entered carrier
names and tracking numbers, saved-search text, watchlist names, rating comments,
dispute statements and chat previews are untrusted content.
Each activity response reports `truncated` when output limits shorten text or
item counts.

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

Only that short-lived delegated API token is used for `GET mcp/account`,
`GET mcp/listings`, `GET mcp/listings/{listing_id}` and the activity reads
beneath `mcp/account/` relative to the API base. The API must independently enforce
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
