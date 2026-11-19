# Budna MCP

[![build](https://img.shields.io/github/actions/workflow/status/budna-marketplace/mcp/core.yml?branch=main)](https://github.com/budna-marketplace/mcp/actions/workflows/core.yml)
[![crates.io](https://img.shields.io/crates/v/budna-mcp.svg)](https://crates.io/crates/budna-mcp)
[![npm](https://img.shields.io/npm/v/@budna-marketplace/mcp.svg)](https://www.npmjs.com/package/@budna-marketplace/mcp)
[![MCP Registry](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fregistry.modelcontextprotocol.io%2Fv0.1%2Fservers%2Fio.github.budna-marketplace%252Fmcp%2Fversions%2Flatest&query=%24.server.version&label=MCP+Registry&prefix=v&color=blue)](https://registry.modelcontextprotocol.io/?q=io.github.budna-marketplace%2Fmcp)
[![Homebrew](https://img.shields.io/badge/dynamic/regex?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbudna-marketplace%2Fhomebrew-tap%2Fmain%2FFormula%2Fbudna-mcp.rb&search=version+%22%28%5B%5E%22%5D%2B%29%22&replace=v%241&label=Homebrew&logo=homebrew&logoColor=white&color=FBB040)](https://github.com/budna-marketplace/homebrew-tap)

Discover new and pre-owned items on Budna that fit your needs and budget. Compare options, check condition and compatibility, and plan pickup or delivery before you decide.

Use Budna from any compatible MCP client. Explore the public marketplace with no account or API key, or optionally connect your Budna account so your AI assistant can read your own account, listings, bids, offers, watchlists, saved searches, purchases, sales, disputes, messages, payments, notifications and ratings, with the permissions you choose. Public Explore and Account Read are read-only. An operator can enable account actions separately. The original pilot saves or removes a listing in a private watchlist. The account-actions profile adds the independently consented tools listed below.

## Quick start

### Shell (macOS / Linux)

Install a prebuilt binary without Node.js or Rust:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/budna-marketplace/mcp/releases/latest/download/budna-mcp-installer.sh | sh
```

### PowerShell (Windows)

Run in PowerShell 5.1 or newer:

```powershell
irm https://github.com/budna-marketplace/mcp/releases/latest/download/budna-mcp-installer.ps1 | iex
```

Restart your terminal if the installer adds its directory to PATH, then check
`budna-mcp --version`.

### npm / npx

Requires [Node.js](https://nodejs.org/).

```bash
npm install --global @budna-marketplace/mcp
```

For clients that launch the server through npx:

```text
command: npx
arguments: --yes @budna-marketplace/mcp
```

### Homebrew

```bash
brew install budna-marketplace/tap/budna-mcp
```

Homebrew installs Rust automatically and builds from source.

### Cargo

You need [Rust 1.88 or newer](https://rustup.rs/).

Install the latest published release:

```bash
cargo install budna-mcp --locked
```

To install from a local checkout:

```bash
cargo install --path crates/budna-mcp --locked
```

### Add to Codex

On macOS or Linux:

```bash
codex mcp add budna -- "$(command -v budna-mcp)"
codex mcp get budna
```

On Windows PowerShell:

```powershell
$budna = (Get-Command budna-mcp).Source
codex mcp add budna -- $budna
codex mcp get budna
```

Start a new Codex task after adding the server, then ask:

> Use Budna to find public camera listings.

The [Codex MCP documentation](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) covers project-scoped configuration and other client options.

### Add to another stdio MCP client

Configure the absolute path returned by `command -v budna-mcp` (or `(Get-Command budna-mcp).Source` on Windows) with no arguments:

```text
command: /absolute/path/to/budna-mcp
arguments: none
```

For JSON-config clients:

```json
{
  "mcpServers": {
    "budna": {
      "command": "/absolute/path/to/budna-mcp",
      "args": []
    }
  }
}
```

## What you can do

Public tools work with no account in the local `budna-mcp` binary and at `https://api.budna.se/mcp`. Account tools require an authenticated account resource and the corresponding permission. By default the account connection includes both the public marketplace and your own data. Self-hosted deployments can serve account tools alone. [Account configuration](docs/account-access.md#deployment-configuration)

| Tool | Access | What it does |
| --- | --- | --- |
| `search_listings` | Public | Search and filter public marketplace listings |
| `get_listing` | Public | Get one public listing, including Budna's next minimum bid when available for an active auction |
| `get_listing_attributes` | Public | Get a listing's structured public attributes |
| `get_listing_related` | Public | Find public listings related to a listing |
| `get_seller_listings` | Public | List a seller's public listings |
| `browse_listings` | Public | Browse featured listings, the newest listings or one brand's listings |
| `get_categories` | Public | Browse the category tree |
| `get_category_filters` | Public | Get the filters available in a category |
| `get_filter_options` | Public | Get the options for one filter |
| `get_brands` | Public | List brands with their listing counts, or get one brand |
| `get_locations` | Public | List the regions and municipalities search can filter by in Sweden or Norway, with listing counts |
| `get_search_suggestions` | Public | Get listing and category suggestions for partly typed text |
| `get_public_seller_profile` | Public | Get a seller's public profile by public seller ID or exact lowercase public username |
| `get_seller_ratings` | Public | Get a seller's rating statistics and the ratings they received, with stars, comment and the rater's public username |
| `get_listing_bid_summary` | Public | Get a listing's bid count and current price, without bidder identities |
| `get_listing_bid_history` | Public | Get a listing's auction rounds and bid history, with bidders shown as Bidder 1, Bidder 2 and so on, as in the app. With your account connected and the Bids permission, your own bids show as You |
| `get_public_ratings_summary` | Public | Get a listing's aggregate public rating signals |
| `get_listing_questions` | Public | Read a listing's public questions and chat, with authors by public username |
| `browse_collections` | Public | Browse collections, a seller's listings under one banner: the live ones or one seller's |
| `get_collection` | Public | Get one collection and a page of its listings |
| `browse_live_streams` | Public | List live shopping streams on air now or coming up |
| `get_live_stream` | Public | Get one live stream, with the listing on stage, the auction running and the items to come |
| `get_shipping_rates` | Public | Get package sizes and domestic shipping prices per carrier |
| `get_my_account` | Account permission | Read your account ID, account type and display name |
| `get_my_profile` | Profile permission | Read your profile, level and badges, verification, account standing, subscription and onboarding steps |
| `get_my_achievements` | Profile permission | Read your achievements and your progress towards them |
| `get_my_referral` | Profile permission | Read your referral code, the referrals that count and your rewards, never who you referred |
| `get_my_legal_documents` | Profile permission | Read which version of each legal document you accepted and whether a newer one awaits you |
| `list_my_listings` | Listings permission | List your own listings, including drafts, with title, status, type and update time |
| `get_my_listing` | Listings permission | Read owner-visible details of one of your listings, including drafts and your reserve price |
| `list_my_bids` | Bids permission | List your bids with amount, time and status (valid, outbid, won or cancelled), or only the ones currently winning |
| `list_my_auto_bids` | Bids permission | List your automatic bids with the maximum you set and their status |
| `list_my_offers` | Offers permission | List offers you sent or received, with the other party's public username, amount, counter-offer, status, who acted last and expiry |
| `list_my_second_chance_offers` | Offers permission | List second-chance offers you received after an auction you bid on |
| `get_my_offer_history` | Offers permission | Read each step of one offer's negotiation: who acted, the action and the amount |
| `list_my_watchlists` | Watchlist read permission | List your watchlists with names, item counts and visibility when available |
| `set_my_watchlist_membership` | Private pilot · `watchlist:write` | Save or remove one listing in one private watchlist you own |
| `create_my_private_watchlist` | Management profile · `watchlists:write` | Create one private watchlist |
| `rename_my_private_watchlist` | Management profile · `watchlists:write` | Rename one private, non-default watchlist you own |
| `delete_my_private_watchlist` | Management profile · `watchlists:write` | Permanently delete one private, non-default watchlist and its memberships |
| `get_my_listing_draft_eligibility` | Draft pilot · `profile:read` | Check seller verification and open the appropriate Budna verification page |
| `get_my_publication_readiness` / `publish_my_quoted_draft` | Account actions · `selling:read` / `listingpublication:write` | Review eligibility, upfront costs and estimated proceeds, then publish an eligible draft immediately |
| `get_listing_creation_rules` / `list_listing_creation_options` | Account actions · `listings:read` | Read a category’s authoring fields, required values, conditions, formats and revision. Page or search the choices allowed in that category. |
| `get_my_draft_authoring` | Account actions · `listings:read` | Read the current fields, revision, category attributes, media count and listing VAT |
| `edit_my_listing_draft` / `replace_my_draft_attributes` | Account actions · `draftauthoring:write` | Edit reviewed fields or intentionally replace the full category attribute set |
| `set_my_draft_vat` / `discard_my_listing_draft` | Account actions · `draftauthoring:write` | Change an eligible company draft’s VAT treatment or discard one reviewed draft |
| `create_my_similar_draft` | Account actions · `draftauthoring:write` | Copy supported fields from one reviewed listing into a verified seller’s unpublished draft |
| `create_my_listing_draft` | Draft pilot · `listingdrafts:write` | Save one unpublished initial draft as the verified seller |
| `get_my_notification_read_batch` | Account actions · `notifications:read` | Preview up to 50 eligible unread ordinary notifications |
| `mark_my_notifications_read` | Account actions · `notificationstate:write` | Acknowledge the ordinary notification IDs the customer selected |
| `get_my_saved_search_authoring` | Account actions · `watchlist:read` | Read current saved-search usage, limit and supported alert frequencies |
| `create_my_saved_search` | Account actions · `savedsearches:write` | Save a named search with validated criteria |
| `update_my_saved_search` | Account actions · `savedsearches:write` | Edit selected criteria, change frequency, or pause or resume alerts using the observed revision |
| `delete_my_saved_search` | Account actions · `savedsearches:write` | Delete one owned saved search at its observed revision |
| `get_my_message_recipient`, `get_my_conversation_audience` | Account actions · `messages:read` | Resolve the public recipient or the other participant before sending |
| `get_my_message_authoring` | Account actions · `messages:read` | Read the message revision and whether the sender can still edit it |
| `start_my_conversation`, `send_my_message`, `reply_to_my_message` | Account actions · `messages:write` | Deliver one requested private message to the confirmed participant |
| `edit_my_message` | Account actions · `messages:write` | Replace the text of one message you sent at its observed revision |
| `list_my_room_reaction_targets` | Account actions · `chatrooms:read` | Read available post and bid reaction targets, counts and your selections |
| `set_my_room_reaction` | Account actions · `roomreactions:write` | Add or remove one emoji on an eligible public post or bid |
| `report_live_chat_post` | Account actions · `livereports:write` | File a report about one live shop post |
| `list_my_room_moderation_participants` | Account actions · `roommoderation:read` | Read eligible participants and ban scope in a room you own |
| `set_my_room_message_pinned`, `hide_my_room_message` | Account actions · `roommoderation:write` | Pin, unpin or hide a selected public post |
| `report_my_room_participant`, `ban_my_room_participant`, `unban_my_room_participant` | Account actions · `roommoderation:write` | Moderate one participant where Budna permits that seller action |
| `get_my_chat_message` | Account actions · `messages:read` | Read the visible message in the selected public room before replying |
| `get_my_chat_posting` | Account actions · `messages:read` | Check eligibility, audience and reply targets in one public room |
| `post_public_chat_message` | Account actions · `publicchat:write` | Post or reply in the specified public listing, showcase or live room |
| `get_my_message_reaction` | Account actions · `messages:read` | Read your reaction and aggregate counts for one visible message |
| `set_my_message_reaction` | Account actions · `messagereactions:write` | Select or clear your one reaction to a private message |
| `get_my_conversation_settings` | Account actions · `messages:read` | Read your pin, archive and mute state without sending a read receipt |
| `set_my_conversation_setting` | Account actions · `inbox:write` | Pin, archive or mute one conversation for yourself, or reverse that setting |
| `hide_my_message_copy` | Account actions · `inbox:write` | Hide your own copy of one message |
| `get_my_listing_chat_notification_setting` | Account actions · `messages:read` | Read your notification preference for an eligible listing chat |
| `set_my_listing_chat_muted` | Account actions · `inbox:write` | Mute or unmute a listing chat for your account |
| `get_my_profile_authoring` | Account actions · `profile:read` | Read editable profile fields, their visibility and the revision |
| `update_my_public_profile` | Account actions · `profile:write` | Change the display name or bio using the current profile revision |
| `get_my_following_state` | Account actions · `following:read` | Resolve a public handle to a stable account ID and check the follow relationship |
| `set_my_following` | Account actions · `following:write` | Follow or unfollow the one public account that matches both supplied identifiers |
| `list_my_personal_blocks` | Account actions · `personalblocks:read` | List public accounts this customer has blocked |
| `set_my_personal_block` | Account actions · `personalblocks:write` | Set or remove a block on one public account |
| `set_my_live_reminder` | Account actions · `livereminders:write` | Set or remove a personal reminder for one live session |
| `get_budna_account_destination` | Account actions · connected account | Open the Budna web area for verification, media, security, wallet, company or other first-party tasks |
| `get_my_action_status` | Account actions, management or draft profile · original action permission | Check one receipt on the connection that submitted it |
| `list_my_watchlist_listings` | Watchlist permission | List the listings saved in one of your watchlists |
| `list_my_saved_searches` | Watchlist permission | List your saved searches with their criteria, alerts and unseen matches |
| `list_my_saved_search_matches` | Watchlist permission | List the listings one of your saved searches matches now |
| `list_my_live_reminders` | Watchlist permission | List the live streams you asked to be reminded about |
| `list_my_orders` | Orders permission | List your purchases or sales with the other party's public username, payment, delivery and return status, carrier, tracking number and, for sales, your payout |
| `get_my_order` | Orders permission | Read one purchase or sale in detail: tracking events, return, refund and dispute status, and invoice numbers |
| `list_my_disputes` | Orders permission | List disputes about your purchases and sales, with your side, the reason and status |
| `get_my_dispute` | Orders permission | Read one dispute with both parties' statements and the settlement offers |
| `list_my_order_groups` | Orders permission | List your combined shipments, several orders sent as one parcel, with their amounts, carrier and deadlines |
| `get_my_order_group` | Orders permission | Read one combined shipment with its orders, amounts, carrier and deadlines |
| `list_my_notifications` | Notifications permission | List your notifications by type, read state and related listing, order, offer or saved search |
| `get_my_notification_preferences` | Notifications permission | Read your notification settings: channels, categories and quiet hours |
| `get_my_order_rating_eligibility` | Account actions · `orders:read` | Preview the counterparty and rating eligibility for one owned order |
| `prepare_my_bid` | Account actions · `bids:read` | Prepare one nonbinding, short-lived quote in SEK for the current auction round |
| `submit_my_quoted_bid` | Account actions · `bids:write` | Submit the exact quoted bid under the customer’s connected-app approval policy |
| `get_my_automatic_bid_state` | Account actions · `bids:read` | Read the customer’s own maximum and its current revision for one auction |
| `prepare_my_automatic_bid_maximum` | Account actions · `bids:read` | Quote a private standing maximum and show the effect of an immediate bid |
| `get_my_order_closure` | Account actions · `orders:read` | Check a personal order, its payment deadline and its closure or relist status |
| `prepare_my_order_closure` | Account actions · `orders:read` | Review a permitted cancellation or overdue seller close, fees, refund and stock effects |
| `submit_my_quoted_order_closure` | Account actions · `orderclosures:write` | Request approval in Budna to close the reviewed order |
| `prepare_my_closed_order_relist` | Account actions · `orders:read` | Review the original price and duration to relist a closed unpaid sale |
| `relist_my_quoted_closed_order` | Account actions · `ordersrelist:write` | Publish an eligible listing after closing its unpaid order |
| `get_my_binding_checkout` | Account actions · `orders:read` | Read an existing Buy Now checkout, its delivery-selection status and any linked order |
| `prepare_my_binding_checkout_order` | Account actions · `orders:read` | Review quantity, delivery, all-in price, deadlines, unpaid fee and auction closure |
| `submit_my_quoted_checkout_order` | Account actions · `checkoutorders:write` | Request approval in Budna to create one binding unpaid order |
| `get_my_second_chance_sale` | Account actions · `orders:read` | Check whether an unpaid auction order you sold can start a second-chance offer or cancel an active one |
| `prepare_my_second_chance_sale` | Account actions · `orders:read` | Prepare the price and deadlines to start or cancel one cascade |
| `submit_my_quoted_second_chance_sale` | Account actions · `secondchances:write` | Starting a cascade requires approval in Budna |
| `get_my_second_chance_invitation` | Account actions · `offers:read` | Look up your active, personal invitation from a received second-chance offer |
| `prepare_my_second_chance_answer` | Account actions · `offers:read` | Review one invitation and its fixed price, fees, response window and payment deadline before accepting or passing on it |
| `submit_my_quoted_second_chance_answer` | Account actions · `secondchanceanswers:write` | Accepting a second-chance invitation requires approval in Budna and creates a binding unpaid order |
| `prepare_my_offer_change` | Account actions · `offers:read` | Review the exact amount, units, parties’ standing proposals and order terms |
| `submit_my_quoted_offer_change` | Account actions · `offers:write` | Make, raise, counter, lower, accept, reject or withdraw the reviewed proposal |
| `set_my_quoted_automatic_bid_maximum` | Account actions · `autobids:write` | Authorize future bids up to the quoted maximum under the customer’s approval policy |
| `pause_my_automatic_bid` | Account actions · `autobids:write` | Pause future automatic bids for the reviewed instruction |
| `get_my_order_dispute_options` | Account actions · `orders:read` | Check the party’s remedies, current case, outstanding discount and decision deadline |
| `open_my_order_dispute`, `add_my_dispute_statement` | Account actions · `disputecases:write` | Open a case or send the party’s exact statement to the parties and Budna |
| `request_withdraw_my_dispute` | Account actions · `disputecases:write` | Request first-party confirmation to close the claimant’s dispute |
| `decline_my_dispute_settlement`, `withdraw_my_settlement_offer` | Account actions · `disputesettlements:write` | Decline a received discount or withdraw the seller’s outstanding proposal |
| `propose_my_dispute_settlement`, `accept_my_dispute_settlement` | Account actions · `disputesettlements:write` | Request confirmation in Budna of the exact discount or its acceptance |
| `rate_my_order_counterparty` | Account actions · `ratings:write` | Submit the customer’s stars and chosen comment or star-only rating |
| `list_my_ratings` | Ratings permission | List ratings you received or gave, with the other party's public username, stars, comment and listing |
| `get_my_money_summary` | Payments permission | Read your money totals per currency: proceeds pending, on the way, paid out, on hold or in balance, and payments made and refunded |
| `list_my_payment_activity` | Payments permission | List your payment activity, newest first, with amount, kind and status |
| `list_my_payouts` | Payments permission | List your recent payouts of sale proceeds |
| `list_my_debts` | Payments permission | List your debts to Budna, such as fees for unpaid orders |
| `get_my_credits` | Payments permission | Read your priority and boost credit balances |
| `get_my_payout_setup` | Payments permission | Read whether you are set up to receive sale proceeds, without bank details |
| `list_my_conversations` | Messages permission | List your conversations with the other person's public username, latest message and unread count |
| `get_my_conversation` | Messages permission | Read the messages in one of your conversations |
| `list_my_listing_chats` | Messages permission | List the listing chats you take part in, with the latest message and unread count |
| `search_my_messages` | Messages permission | Search your messages for a word or phrase |
| `get_my_seller_insights` | Selling permission | Read views and saves of your listings over 7, 30 or 90 days, your top listings and priority credits |
| `list_my_boosts` | Selling permission | List the boosts you bought, with status, price and dates |
| `get_my_listing_promotion` | Selling permission | Read one listing's boosts, search priority, video processing status and, for a company, its VAT treatment |
| `change_my_shipping_defaults` | Account actions · `shippingdefaults:write` | Change carrier or combined-shipping defaults for future listings |
| `get_my_shipping_rules` / `list_my_shipping_rule_listings` | Account actions · `shippingrules:read` | Read kinds of goods, parcel capacities and paged listing assignments |
| `change_my_shipping_rule` | Account actions · `shippingrules:write` | Apply one reviewed change to a kind, listing assignment or parcel |
| `get_my_fulfillment_terms` | Account actions · `fulfillmentreview:read` | Review the orders affected, funded balance, payout timing and receipt-related dispute consequences |
| `update_my_seller_fulfillment` | Account actions · `sellerfulfillment:write` | Record actual dispatch or handover, or update the tracking for one eligible sale |
| `book_my_order_shipment` | Account actions · `carrierbookings:write` | Book the saved carrier for an eligible funded order or its combined parcel |
| `confirm_my_order_receipt` | Account actions · `buyerreceipts:write` | Request approval in Budna to confirm receipt of a shipped order or in-person pickup |
| `get_my_return_terms` | Account actions · `returnreview:read` | Check return rights, refundable amount, postage, deadlines and refund route on a participating order |
| `change_my_order_return` | Account actions · `returns:write` | Request a withdrawal, agree a return or acknowledge arrival through Budna approval, or submit tracking or call off an eligible return as buyer |
| `get_my_combined_parcel_terms` | Account actions · `parcelreview:read` | Review compatible unpaid orders or a parcel you participate in, including all member terms |
| `request_my_combined_parcel` | Account actions · `parcelrequests:write` | Request buyer confirmation on Budna to combine the reviewed orders |
| `change_my_combined_parcel` | Account actions · `parcelmanagement:write` | Declare or decline a parcel as seller, or dissolve it as buyer |
| `get_my_stream_authoring` | Account actions · `streamauthoring:read` | Review an owned target or an existing stream, its schedule and announced lineup |
| `change_my_stream_plan` | Account actions · `streamplans:write` | Schedule, reschedule or cancel one stream, or add, remove or move one announced shop listing |
| `get_my_showcase_authoring` | Account actions · `showcaseauthoring:read` | Review seller verification, a showcase, affected listing deadlines and related streams |
| `change_my_showcase` | Account actions · `showcases:write` | Create, edit or delete a draft, add one reviewed listing, publish or cancel |
| `prepare_my_listing_change` | Account actions · `selling:read` | Review terms, fees and watcher notifications for an active listing edit, extension, relist or deletion |
| `apply_my_quoted_listing_change` | Account actions · `listingmanagement:write` | Apply the seller’s reviewed change to one current listing revision |
| `get_my_listing_inventory` | Account actions · `listings:read` | Review available stock, checkout reservations and open offers on an owned listing |
| `set_my_listing_stock` / `record_my_listing_sold_elsewhere` | Account actions · `inventory:write` | Set an eligible shelf count or record a reviewed outside sale |
| `get_my_shipping_settings` | Selling permission | Read your default carriers and combined-shipping choice |
| `list_my_showcases` | Selling permission | List your collections, including drafts |
| `list_my_live_sessions` | Selling permission | List your live streams with items sold and revenue |
| `list_my_listing_tags` | Selling permission | List scans and saves of your in-store listing tags |
| `list_my_following` | Following permission | List who you follow, by public username |
| `list_my_followers` | Following permission | List who follows you, by public username |
| `list_my_companies` | Company permission | List your companies, your role in each and invitations sent to you |
| `get_my_company` | Company permission | Read one of your companies, its members by public username and its invoicing status |

In compatible MCP Apps clients you can also browse interactive listing cards and explicitly ask the AI to compare two to four selected listings.

The pilot requires an operator-enabled deployment, a supported API and explicit watchlist write consent. The default profiles continue to serve reads. [Private watchlist pilot](docs/account-access.md#private-watchlist-pilot)

### Permissions you control

- When you connect an app, Budna shows exactly what it asks for and you choose which permissions to grant: **Account** (`account:read`), **Listings** (`listings:read`), **Bids** (`bids:read`), **Offers** (`offers:read`), **Watchlists and saved searches** (`watchlist:read`), **Orders** (`orders:read`), **Notifications** (`notifications:read`), **Ratings** (`ratings:read`), **Payments** (`payments:read`), **Messages** (`messages:read`), **Profile** (`profile:read`), **Selling** (`selling:read`), **Following** (`following:read`) and **Company** (`company:read`).
- Account tools name the other buyer or seller only by the public username the app shows, and never reveal who bid on a listing. Addresses, contact details and notification text are never included; message text is shared only with the Messages permission, and dispute statements only with the Orders permission.
- Under Account → Security → Connected apps you can remove a permission or disconnect an app at any time. Removing a permission takes effect immediately, including for access the app already holds; restoring it requires approving the app again.
- A connection lasts at most seven days before you approve it again.
- Disconnecting stops future access, but cannot recall information already shared with the AI service.

### Where Budna MCP is going

In deployments with the account-actions profile, customers can organize private lists and searches, create a verified unpublished draft, send messages, manage supported room activity and submit their own order ratings. Operators select the families their Budna API supports. The remaining seller lifecycle, bids, offers, checkout and after-sale actions are being implemented in waves with their own eligibility and confirmation rules. The handoff records each family’s release and verification status. [Account action plan](docs/account-write-plan.md)

## Connect your Budna account

Add `https://api.budna.se/account/mcp` to your AI client as a remote (Streamable HTTP) MCP server. The client opens your browser to sign in to Budna at `https://connect.budna.se` and shows which permissions it requests; you choose which to grant. See [Permissions you control](#permissions-you-control) for reducing or removing access later.

| Client | Setup |
| --- | --- |
| Claude Code | `claude mcp add --transport http budna https://api.budna.se/account/mcp`, then `/mcp` to authenticate |
| Codex | `codex mcp add budna-account --url https://api.budna.se/account/mcp`, then `codex mcp login budna-account` |
| ChatGPT | Add a custom connector with the URL, OAuth client ID `budna-chatgpt` and no client secret |
| Claude (web and desktop) | Add a custom connector with the URL and OAuth client ID `budna-claude` |
| Visual Studio Code | Add the URL as an HTTP MCP server |
| goose | Add the URL as a remote extension |
| Gemini CLI | Add the URL with `oauth.clientId` set to `budna-gemini-cli` |
| Cursor | Add the URL with OAuth client ID `budna-cursor` |

Other clients are not accepted yet. See [Account access](docs/account-access.md) for the full permission model and the data each tool returns.

## Deployment configuration

Budna MCP uses production endpoints by default. For local testing or a custom deployment, see the [endpoint configuration guide](docs/mcp-apps.md#environment-endpoints).

## Interactive cards

The Marketplace Explorer is embedded in the binary. Clients that support the [MCP Apps extension](https://modelcontextprotocol.io/extensions/apps/overview) can render listing cards, details, related/seller listings, and comparison selection. Other clients receive the same text and structured JSON fallback. See the [extension support matrix](https://modelcontextprotocol.io/extensions/client-matrix) for client coverage.

See [the Marketplace Explorer guide](docs/mcp-apps.md) for local HTTP testing, temporary-tunnel safety, privacy and attribution behavior, UI development, and release checks.

The project is licensed under the [MIT License](LICENSE).
