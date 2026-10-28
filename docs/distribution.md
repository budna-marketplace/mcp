# Distribution and discovery

Investigation date: 2026-10-28. Recommendations and submission terms below
should be rechecked before use. No directory submissions or new package-manager
listings have been made as part of this investigation.

## Release channels

The existing release workflow publishes Cargo crates, `@budna-marketplace/mcp`
on npm, and the `budna-marketplace/homebrew-tap` formula. GitHub releases contain
five binary targets: macOS ARM64/x86-64, Linux GNU ARM64/x86-64, and Windows
MSVC x86-64. npm already distributes those binaries, so additional channels
should add installation convenience or discovery rather than duplicate builds.

Official MCP Registry publishing is configured in this checkout:

- `server.json` identifies `io.github.budna-marketplace/mcp` and its npm package.
- Distribution adds the matching `mcpName` to the canonical `budna-mcp.tgz`.
- The release workflow publishes Registry metadata after npm succeeds, using
  GitHub OIDC. The Registry stores metadata; npm stores the installable package.
- The `registry` deployment environment uses the same reviewer and `v*` tag
  restriction as the existing release environments; no Registry secret is needed.
- Bump both versions in `server.json` with every package release. Start with a
  fresh release: published npm version 0.3.2 lacks the required marker, and
  existing package versions and release tags must not be replaced.

See the [Registry publishing guide](https://modelcontextprotocol.io/registry/quickstart)
and the repository's release skill for the full release sequence. This local
preparation does not mean the Registry entry has been published.

## Native script installers

cargo-dist generates `budna-mcp-installer.sh` and `budna-mcp-installer.ps1`
with each release. The release draft includes both scripts, their SHA256 files,
and the matching native archives. No Node.js or Rust installation is required.

After the first release containing these assets, install on macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/budna-marketplace/mcp/releases/latest/download/budna-mcp-installer.sh | sh
```

On Windows, run in PowerShell 5.1 or newer:

```powershell
irm https://github.com/budna-marketplace/mcp/releases/latest/download/budna-mcp-installer.ps1 | iex
```

Restart the terminal if the installer adds its directory to PATH. For a pinned
release, replace `latest/download` with `download/v<VERSION>` in the URL.

Distribution CI installs from local HTTP archives into an isolated directory,
without changing profiles or PATH, then checks `--version` and exercises MCP
initialize, tools-list, and tools-call against synthetic API mocks on every
supported target. The original generated scripts are uploaded unchanged.

Recommended next additions, all still planned:

| Order | Channel | Smallest useful setup | Release consideration |
| --- | --- | --- | --- |
| 1 | [WinGet](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest) | Manifest for the existing Windows archive; submit to the community repository. | Generate a versioned manifest and update PR from each release. Review can delay availability. |
| 2 | [Scoop](https://github.com/ScoopInstaller/Scoop/wiki/Creating-an-app-manifest) | Own bucket with a manifest for the existing Windows binary. | Update version, download URL, and checksum; bucket access and Windows install testing are needed. |
| 3 | [Docker / GHCR](https://docs.docker.com/ai/mcp-catalog-and-toolkit/) | Publish a container image with the binary and stdio entry point. | Image publishing is separate from acceptance into Docker's curated MCP Catalog. Add container smoke tests and image maintenance. |
| 4 | [MCPB](https://github.com/modelcontextprotocol/mcpb) | Bundle the existing native binaries for desktop installation. | Validate supported desktop platforms and bundle metadata; [Smithery](https://smithery.ai/docs/build/publish) accepts local MCPB releases. |

Defer Chocolatey, PyPI/uvx wrappers, apt/rpm repositories, Nixpkgs, and AUR until
there is demonstrated demand. They introduce more packaging, credentials,
review, or platform maintenance; Cargo, npm, Homebrew, and native archives
already provide an installation route for the current platforms.

Publishing a release makes a new version available. It does not update running
MCP processes. Installed users still need their package manager's update or
reinstall step and a server restart; pinned versions need an explicit change.
Do not promise uniform update behavior across MCP clients or directory listings.

## Discovery submissions

Prioritize the Official MCP Registry, then these repository-based listings.
Directory listings are discovery links, not additional package registries.

| Channel | Submission route | Current requirements and limits |
| --- | --- | --- |
| [punkpeye/awesome-mcp-servers](https://github.com/punkpeye/awesome-mcp-servers/blob/main/CONTRIBUTING.md) | GitHub PR under E-Commerce. | Public installable repository, concise description, existing format, alphabetical order. A repository link normally survives new releases. |
| [Glama](https://glama.ai/mcp/faq) | Servers page → Add Server. | GitHub URL, display name, description; automated quality checks. Source listing does not require hosted Budna MCP. Its [documented sources](https://glama.ai/mcp/reference) include the official Registry and punkpeye's list. |
| [MCP.Directory](https://mcp.directory/submit) | Submit form. | GitHub URL; npm name, description of at most 100 characters, and email optional. Advertises review within 24 hours and official Registry discovery. |
| [mcpservers.org](https://mcpservers.org/submit) | Submit form; also the route for [wong2's list](https://github.com/wong2/awesome-mcp-servers). | Name, category, description, project URL, contact email; Registry name optional. Free review within two weeks; optional $39 expedited review. Wong2 does not accept PRs. |
| [MCP Market](https://mcpmarket.com/submit) | GitHub repository form. | Repository and email. Free queue advertises 4–6 weeks; optional $29 listing within 24 hours. |
| [Cursor Directory](https://cursor.directory/plugins/new) | Submit a plugin after validating its format. | Sign in and supply repository. The [directory's guide](https://github.com/cursor/community-plugins#contributing) detects MCP from `.mcp.json`; its linked [Agent Plugins standard](https://agent-plugins.org/) now uses `plugin.json` and `mcp.json`. Verify compatibility before adding files. |

These are the current exclusions and later opportunities:

- [PulseMCP](https://www.pulsemcp.com/submit) has paused submissions and listing
  changes. Its notice says official Registry entries will be imported when it
  reopens; there is no reopening date.
- [appcypher/awesome-mcp-servers](https://github.com/appcypher/awesome-mcp-servers)
  is archived and read-only.
- [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers)
  houses reference implementations and directs discovery to the Registry.
- [mcp.so](https://mcp.so/submit) currently shows a $39 submission fee; no free
  submission path was verified. Do not pay for expedited listings by default.
- [Smithery](https://smithery.ai/docs/build/publish) accepts a local MCPB bundle
  or a hosted HTTPS Streamable HTTP endpoint. Plain npm publication alone does
  not establish either route. Remote listings require an actual deployment.

Maintainer review, directory indexing, and third-party queues can lag a release.
Only promise automated updates for channels whose publishing jobs are configured
and verified. Check for duplicate listings before submitting; no Budna entry was
found in the current punkpeye or wong2 READMEs, while dynamic directory status
remains unconfirmed.

## Public submission copy

```text
Name: Budna MCP
Repository: https://github.com/budna-marketplace/mcp
npm: @budna-marketplace/mcp
Registry: io.github.budna-marketplace/mcp
Description: Explore public Budna marketplace listings, categories, seller profiles, ratings, and bid summaries.
Transport: stdio
Install command: npx --yes @budna-marketplace/mcp
Authentication: No Budna account or API key required for public Explore.
Category: E-Commerce / Search
Language: Rust
Platforms: macOS, Linux, Windows
```

Suggested punkpeye entry, to adapt to its current formatting when submitting:

```markdown
- [budna-marketplace/mcp](https://github.com/budna-marketplace/mcp) 🎖️ 🦀 ☁️ 🍎 🪟 🐧 - Explore public Budna marketplace listings, categories, seller profiles, ratings, and bid summaries without an account or API key.
```

Its cloud icon denotes access to remote APIs, not a hosted MCP endpoint. Add a
Glama badge only after verifying the listing and badge URL. Use an approved
public contact address when a form requires one; no ownership metadata has been
added based on an assumed maintainer identity.
