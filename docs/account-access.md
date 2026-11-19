# Account access

Public Explore is the default profile. Account Read adds delegated access to the
customer’s own data. The private watchlist pilot adds one separately consented
action on the same account resource. Customers choose which permissions to grant
and can revoke them in Connected apps.

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

`AccountMcpServer::new` serves account reads. Adding `with_public` also serves
the public Explore tools and Marketplace Explorer resources on `/account/mcp`,
so a connected client reads public and own data through one connection. Public
tools need no delegated permission and use the credential-free public client.
With the `bids:read` permission, `get_listing_bid_history` on the account
resource labels the connected account's own bids `You`. Other bidders retain
their numbered labels. On the public resource every bidder has a numbered label.

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

The standalone Account Read profile includes public Explore by default. Set
`BUDNA_MCP_ACCOUNT_PUBLIC_EXPLORE=false` or pass
`--account-public-explore false` to serve only account tools on that resource.
This also removes the embedded Marketplace Explorer from that connection.

Provision `BUDNA_MCP_SERVICE_SECRET` separately through the deployment's secret
store. It is a dedicated service credential, never a customer password,
marketplace session or signing key. No command-line secret option is provided.
The account profile fails startup when required configuration is missing.

The operator must register supported OAuth clients and their exact HTTPS
redirects with the authorization service. Discovery is published at
`/.well-known/oauth-protected-resource/account/mcp`. The authorization service
must advertise its authorization-code/PKCE S256 flow and support the customer's
browser sign-in, explicit consent, token rotation and per-application revocation.

## Private watchlist pilot

Run the standalone server with `BUDNA_MCP_PROFILE=account-watchlist-pilot`
and the same Streamable HTTP, issuer, resource and service credential settings
as Account Read. The pilot starts with account tools only. Set
`BUDNA_MCP_ACCOUNT_PUBLIC_EXPLORE=true` to include public Explore. Enable the
pilot only for designated clients on a private deployment with corresponding
Budna API support. Deploying the binary alone does not grant write access.

| Tool | Permission | Behavior |
| --- | --- | --- |
| `set_my_watchlist_membership` | `watchlist:write` | On the customer's request, set `saved` to true to save a listing or false to remove it from one owned private watchlist. |

Supply a positive integer `watchlist_id`, a canonical `listing_id` and a
boolean `saved`. When the customer names a list, use `list_my_watchlists`
with separately granted `watchlist:read` to resolve its ID. A missing
`is_private` on an older server means visibility is unknown. If several lists
match, ask the customer which one they mean. Reading the current items also
requires `watchlist:read`. The write alone can use IDs the customer supplies.

**Delegated API contract.** Send `PUT mcp/account/watchlists/{watchlist_id}/listings/{listing_id}`
relative to the API base with the delegated bearer token and an
`Idempotency-Key` header containing a canonical lowercase, non-nil UUID.
The gateway creates the key for each tool call. The JSON body accepts only
`{"saved":true}` or `{"saved":false}`. Budna enforces ownership and current
privacy at execution. An unavailable list, including a public or foreign
one, returns HTTP 404. Removing a listing affects only the named list.

A synthetic example of the API response follows. The MCP tool returns the
five fields inside `data` as structured content.

```json
{
  "success": true,
  "data": {
    "action_id": "00000000-0000-4000-8000-000000000001",
    "watchlist_id": 7,
    "listing_id": "listing_00000000-0000-4000-8000-000000000002",
    "saved": true,
    "status": "executed"
  }
}
```

`saved` records the requested state. Only `executed` confirms the action
was applied or already satisfied at execution time. Reusing a key for the
same request returns the recorded outcome, even if someone later changed
the list. Reusing it with different inputs produces a conflict. For
`executing`, `unknown` or a lost response, check the watchlist and action
history in Budna before requesting another change. The gateway never
automatically retries a dispatched write.

The pilot grant covers only the action above. Keep general availability
and any broader watchlist permissions behind separate rollout and consent
review. Use Account Read or Public Explore to turn off the write tool.

## Configuring account actions

Use the HTTP profile `account-actions` to enable multiple independently consented
families on one connection. Set `BUDNA_MCP_WRITE_SCOPES` to the exact
space-separated permissions approved for the deployment. Include `personalblocks:read`, `chatrooms:read` or `roommoderation:read` explicitly when they are needed. For example,
`watchlist:write livereminders:write` enables private listing membership and
personal live reminders. Unknown, duplicate and empty permission lists fail
startup. A client also needs current consent for each tool it invokes.
The standalone profile starts with public Explore off.

`set_my_live_reminder` takes a canonical `session_id` and `reminded` boolean.
Its receipt contains `action_id`, `session_id`, `reminded` and `status`.
Use `list_my_live_reminders` under `watchlist:read` to check the current state.
The write does not join a stream or record audience activity.

Under the `notificationstate:write` deployment permission,
`get_my_notification_read_batch` uses a `notifications:read` grant to return
up to 50 eligible unread IDs and `has_more`. With the separate
`notificationstate:write` grant, send exactly the IDs the customer wants to
acknowledge to `mark_my_notifications_read`. Later arrivals retain their state.
Protected, foreign or unavailable IDs reject the entire request. The receipt
contains `action_id`, `status` and `requested_count`. The count includes
eligible IDs that another client already acknowledged.

Use `get_budna_account_destination` on the combined profile to open the
appropriate Budna web area for a first-party task. This accepts a fixed task
name. The tool constructs the URL from the configured first-party origin and
returns no session material. Verification, payment and connected-app approval
take place in the customer's own browser.

### Saved searches

Enable `savedsearches:write` on the account-actions deployment.
`get_my_saved_search_authoring` uses `watchlist:read` to return usage, the
account limit and allowed frequencies. Resolve category, filter and location
choices using the marketplace reads. For creation, give the new search a
name and supply the criteria. Monetary bounds use exact decimal strings, with SEK for Swedish searches
and NOK for Norwegian searches. An omitted market on creation uses SEK. Frequencies are immediate, daily or weekly.

Read the saved search to obtain its current `updated_at`. Send that value as
`expected_updated_at` to edit or delete. On edit, omitted scalar criteria are
preserved and an explicit null clears the supplied criterion. Set or remove
individual taxonomy filters with entries such as `{"field":"brand","value":null}`.
Changing `active` pauses or resumes alerts. A stale revision requires reading
the search again. Receipts contain `action_id`, `search_id` and `status`.
Browsing and editing leave match acknowledgement in Budna.

### Personal inbox, profile and follows

Enable `inbox:write` for personal conversation settings, hiding a participant’s
own message copy and muting eligible listing chats. Read the current settings
with `messages:read`. Archiving hides a conversation for this participant.
Hiding a message copy leaves the other participant’s copy intact.

Enable `profile:write` for display-name and bio changes. Use
`get_my_profile_authoring` with `profile:read` to obtain the current revision.
Omit fields to preserve them and supply an empty bio to clear it. The existing
visibility setting applies. Manage public city through Budna address settings.

Enable `following:write` for a single public follow or unfollow. Use
`get_my_following_state` under `following:read` to read the exact handle,
stable public ID and relationship. The write requires the same handle and
stable ID. A new follow can notify the other person.

### Personal blocks

With `messages:write`, one customer request can start a private conversation,
send a message, reply to a specific message or replace the text of a message
the customer sent within its edit window. Use the read tools under
`messages:read` to confirm the other participant’s current public username
and stable ID, and the current revision for an edit. Sending can notify the
recipient and reopen the thread. These tools preserve the requested text and
Budna’s moderation rules. To react to a visible private message, grant
`messagereactions:write` separately. Pass one supported emoji or explicitly
pass `null` to remove the customer’s reaction. The reaction read returns the
customer’s choice and aggregate counts.

Room reactions, reports and seller moderation use independent grants.
`chatrooms:read` returns public message and bid IDs, aggregate reaction
counts and the customer’s choices. `roomreactions:write` sets one emoji
to the requested state on one eligible target. A shop-stream report uses
`livereports:write` and the actual message ID. Report receipts acknowledge
filing. Staff determine the outcome.

`roommoderation:read` previews the owner’s eligible participants and
the scope of a ban. `roommoderation:write` permits the applicable pin,
hide, report or ban action. A seller can lift a shop-stream ban across
their live chats. Hiding a room post is permanent through the app's
ordinary controls. These tools accept only the public participant and
post identifiers needed for that action.

For an order rating, use `get_my_order_rating_eligibility` with `orders:read`.
Budna derives the other party and the rating direction from the selected
transaction. Submission needs `ratings:write`, one to five customer-chosen
stars, the explicit marketplace audience and either the customer’s exact
comment or an explicit `null` for a star-only rating. Eligibility depends
on the transaction and which side the customer was on. A qualifying unpaid
auction closure can allow the seller to rate the buyer.

Public posting requires its own `publicchat:write` consent. With
`messages:read`, check the posting rules and available reply targets for the
exact room. To submit, provide its public room ID, the explicit audience
`public`, the text to post and, for a reply, the parent message ID.
Budna rechecks eligibility and moderation rules when it receives the post.

`personalblocks:read` and `personalblocks:write` are independent opt-in
permissions. The read returns only the connected account’s outgoing blocks,
with public IDs and handles. Supply the returned `next_after` for the next
page. Blocking removes reciprocal follows and pauses standing automatic bids
between the accounts. Unblocking restores neither. Explain these effects
before submitting the customer’s request. The receipt contains `action_id`,
`status`, the target `user_id` and the requested `blocked` state.

## Private watchlist management

Select `BUDNA_MCP_PROFILE=account-watchlist-management` on a designated
HTTP deployment. Public Explore on this profile requires explicit opt-in.
Budna advertises enabled permissions in resource discovery. Existing
`watchlist:write` consent covers only listing membership. The customer
selects the separate `watchlists:write` permission to manage their lists.

| Tool | Inputs | Result |
| --- | --- | --- |
| `create_my_private_watchlist` | `name`, a nonblank name of up to 255 UTF-8 bytes without control characters | Creates one private, non-default list. |
| `rename_my_private_watchlist` | `watchlist_id` and `name` | Changes the name of one owned private, non-default list. |
| `delete_my_private_watchlist` | `watchlist_id` | Permanently removes the list and its memberships. Listings saved in other lists stay there. |
| `get_my_action_status` | `action_id` from one receipt | Reads the action on its original connection with the permission that submitted it. |

A management receipt contains `action_id`, `watchlist_id` and `status`.
The list ID may be unknown while creation is unresolved. Only `executed`
confirms completion. To resolve a receipt, the status tool returns the same
action reference, its current state and an optional expiry in epoch milliseconds.
When a receipt is lost, check Connected apps in Budna and read the affected list
before requesting another change. Private list names never appear in receipts.
Public, default, missing and foreign lists return the same unavailable result
for rename and delete. List discovery still requires `watchlist:read`.

## Verified listing drafts

The opt-in HTTP profile `account-listing-draft-pilot` accepts a separate
`listingdrafts:write` grant for creating one initial draft. Its optional
`get_my_listing_draft_eligibility` read uses `profile:read`. The result contains
`seller_verification` and an optional `verification_url` on the configured
first-party web origin. Complete verification in that browser session and check
again. Budna rechecks the seller at the point where it saves the draft.
A connected company verifies its own organization.

`create_my_listing_draft` requires a `market` of `swedish` or `norwegian`
and `currency` of `SEK`. Omit `title` and `description` for a blank draft.
Supplied text must be nonblank, with at most 200 and 10,000 characters
respectively. The server supplies ownership, status and defaults. Add media or
continue editing in Budna. The receipt contains `action_id`, `listing_id`
and `status`. Check `get_my_action_status` on the originating connection
when a receipt is pending or unknown.

## Read permissions and tools

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
| `list_my_watchlists` | `watchlist:read` | Own watchlists with ID, name, default flag, item count, timestamps and optional `is_private` |
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
| `search_my_messages` | `messages:read` | Own conversation messages containing `query`: conversation and message IDs, sender (`me` or `other_party`), a snippet of up to 300 characters and time; continue with `next_page` |
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
for ratings. Message search takes `query`, 3–100 characters after trimming,
and returns `next_page` instead of a total. The order, dispute,
combined-shipment, offer-history and company reads take only a positive
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
dispute statements, chat previews and message snippets are untrusted content.
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
discovery information. When an authenticated caller lacks the permission for a
tool, HTTP 403 names the specific scope it needs. For example, calling
`list_my_watchlists` requests `watchlist:read`. Discovery still lists all
supported scopes.

The resource exchanges an inbound token at the configured authorization
service using the dedicated service credential. The request is JSON
`{"subject_token":"<MCP token>","resource":"<exact resource>"}` to
`mcp/oauth/exchange` beneath the configured API base. The response contains a
separate bearer API token, expiry, scope, subject, grant ID, client ID and
resource. Tokens are never returned as tool output or embedded in resources.

Only that short-lived delegated API token is used for `GET mcp/account`,
`GET mcp/listings`, `GET mcp/listings/{listing_id}` and the activity reads
beneath `mcp/account/` relative to the API base. The API must independently enforce
grant revocation, expiry, scope, account standing and ownership, and must limit
these credentials to explicitly enabled capabilities. The pilot uses the same
credential boundary for its single documented PUT. The MCP token is never forwarded
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
