# Budna MCP

[![build](https://img.shields.io/github/actions/workflow/status/budna-marketplace/mcp/core.yml?branch=main)](https://github.com/budna-marketplace/mcp/actions/workflows/core.yml)
[![crates.io](https://img.shields.io/crates/v/budna-mcp.svg)](https://crates.io/crates/budna-mcp)
[![npm](https://img.shields.io/npm/v/@budna-marketplace/mcp.svg)](https://www.npmjs.com/package/@budna-marketplace/mcp)
[![MCP Registry](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fregistry.modelcontextprotocol.io%2Fv0.1%2Fservers%2Fio.github.budna-marketplace%252Fmcp%2Fversions%2Flatest&query=%24.server.version&label=MCP+Registry&prefix=v&color=blue)](https://registry.modelcontextprotocol.io/?q=io.github.budna-marketplace%2Fmcp)
[![Homebrew](https://img.shields.io/badge/dynamic/regex?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbudna-marketplace%2Fhomebrew-tap%2Fmain%2FFormula%2Fbudna-mcp.rb&search=version+%22%28%5B%5E%22%5D%2B%29%22&replace=v%241&label=Homebrew&logo=homebrew&logoColor=white&color=FBB040)](https://github.com/budna-marketplace/homebrew-tap)

Use Budna from any compatible MCP client. Explore the public marketplace with no account or API key, or optionally connect your Budna account so your AI assistant can read your own account and listings, with the permissions you choose. Budna MCP is read-only: it cannot change marketplace data.

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

### Explore the marketplace (no account)

The local `budna-mcp` binary and the hosted endpoint `https://api.budna.se/mcp` expose 11 public, read-only tools:

- Discover listings, categories, filters, and filter options.
- Inspect listings, attributes, related listings, seller listings, public seller profiles, bid summaries, and rating summaries.
- In compatible MCP Apps clients, browse interactive cards and explicitly ask the AI to compare two to four selected listings.

### Read your own account (optional)

After you connect your Budna account, the hosted account resource adds three read-only tools:

| Tool | Permission | What it reads |
| --- | --- | --- |
| `get_my_account` | Account | Your account ID, account type and display name |
| `list_my_listings` | Listings | Your listings, including drafts, with title, status, type and update time |
| `get_my_listing` | Listings | Owner-visible details of one of your listings, including drafts and your reserve price |

Budna MCP cannot bid, make offers, buy, message, record views, change account settings, or otherwise change marketplace data. The local binary never signs in; account access exists only on the hosted resource and only after you approve it.

## Connect your Budna account

Add `https://api.budna.se/account/mcp` to your AI client as a remote (Streamable HTTP) MCP server. The client opens your browser to sign in to Budna at `https://connect.budna.se` and shows which permissions it requests; you choose which to grant. Review, reduce or disconnect each app at any time under Account → Security → Connected apps. Disconnecting stops future access but cannot recall information already shared with the AI service.

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
