# Budna MCP

[![build](https://img.shields.io/github/actions/workflow/status/budna-marketplace/mcp/core.yml?branch=main)](https://github.com/budna-marketplace/mcp/actions/workflows/core.yml)
[![crates.io](https://img.shields.io/crates/v/budna-mcp.svg)](https://crates.io/crates/budna-mcp)
[![npm](https://img.shields.io/npm/v/@budna-marketplace/mcp.svg)](https://www.npmjs.com/package/@budna-marketplace/mcp)
[![MCP Registry](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fregistry.modelcontextprotocol.io%2Fv0.1%2Fservers%2Fio.github.budna-marketplace%252Fmcp%2Fversions%2Flatest&query=%24.server.version&label=MCP+Registry&prefix=v&color=blue)](https://registry.modelcontextprotocol.io/?q=io.github.budna-marketplace%2Fmcp)
[![Homebrew](https://img.shields.io/badge/dynamic/regex?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbudna-marketplace%2Fhomebrew-tap%2Fmain%2FFormula%2Fbudna-mcp.rb&search=version+%22%28%5B%5E%22%5D%2B%29%22&replace=v%241&label=Homebrew&logo=homebrew&logoColor=white&color=FBB040)](https://github.com/budna-marketplace/homebrew-tap)

Use Budna from any compatible MCP client. Explore the public marketplace with no account or API key, or optionally connect your Budna account so your AI assistant can read your own account, listings, bids, offers, watchlists, saved searches, purchases, sales, notifications and ratings, with the permissions you choose. Budna MCP is read-only: it cannot change marketplace data.

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

Budna MCP is read-only today. Public tools work with no account in the local `budna-mcp` binary and at `https://api.budna.se/mcp`. Account tools are available only on the hosted account resource, after you connect your Budna account and grant the permission they need.

| Tool | Access | What it does |
| --- | --- | --- |
| `search_listings` | Public | Search and filter public marketplace listings |
| `get_listing` | Public | Get one public listing |
| `get_listing_attributes` | Public | Get a listing's structured public attributes |
| `get_listing_related` | Public | Find public listings related to a listing |
| `get_seller_listings` | Public | List a seller's public listings |
| `get_categories` | Public | Browse the category tree |
| `get_category_filters` | Public | Get the filters available in a category |
| `get_filter_options` | Public | Get the options for one filter |
| `get_public_seller_profile` | Public | Get a seller's public profile |
| `get_listing_bid_summary` | Public | Get a listing's bid count and current price, without bidder identities |
| `get_public_ratings_summary` | Public | Get a listing's aggregate public rating signals |
| `get_my_account` | Account permission | Read your account ID, account type and display name |
| `list_my_listings` | Listings permission | List your own listings, including drafts, with title, status, type and update time |
| `get_my_listing` | Listings permission | Read owner-visible details of one of your listings, including drafts and your reserve price |
| `list_my_bids` | Bids permission | List your bids with amount, time and status (valid, outbid, won or cancelled), or only the ones currently winning |
| `list_my_offers` | Offers permission | List offers you sent or received, with amount, counter-offer, status, who acted last and expiry |
| `list_my_watchlists` | Watchlist permission | List your watchlists with their names and item counts |
| `list_my_watchlist_listings` | Watchlist permission | List the listings saved in one of your watchlists |
| `list_my_saved_searches` | Watchlist permission | List your saved searches with their criteria, alerts and unseen matches |
| `list_my_orders` | Orders permission | List your purchases or sales with payment, delivery and return status, carrier, tracking number and, for sales, your payout |
| `list_my_notifications` | Notifications permission | List your notifications by type, read state and related listing, order, offer or saved search |
| `list_my_ratings` | Ratings permission | List ratings you received or gave, with stars, comment and listing |

In compatible MCP Apps clients you can also browse interactive listing cards and explicitly ask the AI to compare two to four selected listings.

Budna MCP cannot currently bid, make offers, buy, message, record views, change account settings, or otherwise change marketplace data.

### Permissions you control

- When you connect an app, Budna shows exactly what it asks for and you choose which permissions to grant: **Account** (`account:read`), **Listings** (`listings:read`), **Bids** (`bids:read`), **Offers** (`offers:read`), **Watchlists and saved searches** (`watchlist:read`), **Orders** (`orders:read`), **Notifications** (`notifications:read`) and **Ratings** (`ratings:read`).
- Account tools never show who the other buyer, bidder or seller is, their addresses or contact details, or the text of messages and notifications.
- Under Account → Security → Connected apps you can remove a permission or disconnect an app at any time. Removing a permission takes effect immediately, including for access the app already holds; restoring it requires approving the app again.
- A connection lasts at most seven days before you approve it again.
- Disconnecting stops future access, but cannot recall information already shared with the AI service.

### Where Budna MCP is going

Budna MCP is growing toward everything you can do in the Budna app and on the web. Next come the remaining reads of your own data, such as messages, payments and payouts, returns and disputes, and your profile. After that come actions such as bidding, making offers, buying, selling and managing your listings, and messaging. Each new capability will arrive behind its own permission that you grant per app, sensitive actions such as bids and purchases will ask for your explicit confirmation, and you can reduce or revoke access at any time. None of these actions are available yet.

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

## Use a non-production environment

Configure these values together so API requests, public listing links, and images stay in the same environment:

| Setting | Purpose |
| --- | --- |
| `BUDNA_API_URL` | Budna API base URL, including `/api/v1` if applicable. |
| `BUDNA_PUBLIC_LISTING_ORIGIN` | Origin used for returned listing links. |
| `BUDNA_IMAGE_ORIGIN` | Origin used for returned image URLs and the App CSP. |

Remote API URLs must use HTTPS; HTTP is accepted only for loopback development servers. Listing and image values must be canonical HTTPS origins with no credentials, path, query, fragment, or wildcard host. The binary does not load `.env` files itself; [`.env.example`](.env.example) is reference material.

For example, register a separate Codex server for another environment:

```bash
codex mcp add budna-nonprod -- /absolute/path/to/budna-mcp \
  --api-url https://api.your-environment.example/api/v1 \
  --public-listing-origin https://marketplace.your-environment.example \
  --image-origin https://images.your-environment.example
```

Equivalent environment variables are supported when your client can provide them.

## Interactive cards

The Marketplace Explorer is embedded in the binary. Clients that support the [MCP Apps extension](https://modelcontextprotocol.io/extensions/apps/overview) can render listing cards, details, related/seller listings, and comparison selection. Other clients receive the same text and structured JSON fallback. See the [extension support matrix](https://modelcontextprotocol.io/extensions/client-matrix) for client coverage.

See [the Marketplace Explorer guide](docs/mcp-apps.md) for local HTTP testing, temporary-tunnel safety, privacy and attribution behavior, UI development, and release checks.

The project is licensed under the [MIT License](LICENSE).
