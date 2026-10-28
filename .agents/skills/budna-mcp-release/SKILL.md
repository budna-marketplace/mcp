---
name: budna-mcp-release
description: Prepare, publish, or troubleshoot Budna MCP releases across Cargo, Homebrew, npm, native installers, and the official MCP Registry using the existing GitHub Actions workflows.
---

# Budna MCP Release

Read root `AGENTS.md`, `Cargo.toml`, and the workflows below before acting.
Paths are relative to the repository root. Preparation stops at a verified
draft unless publication is already authorized by the user.

## Prepare

1. Check remote tags, GitHub releases, and registry versions before choosing a
   new version. Never move or reuse a release tag.
2. Update the workspace version and local dependency versions in `Cargo.toml`,
   workspace entries in `Cargo.lock`, UI `package.json` and `package-lock.json`,
   `appInfo.version` in `ui/marketplace-explorer/src/bridge.ts` and its handshake
   expectation in `ui/marketplace-explorer/test/bridge.test.ts`, and both the
   server and npm package versions in `server.json`.
3. Follow `ui/marketplace-explorer/AGENTS.md` to rebuild the embedded HTML.
   Run root verification and the packaging checks in `.github/workflows/core.yml`.
   Its workspace publish dry run needs the explicit Cargo toolchain shown there;
   changing rustup's default does not override `rust-toolchain.toml`.
4. After the version changes reach `main` and checks pass, create and push a new
   `v<version>` tag at that commit. Tagged source fixes require a fresh version.
5. Dispatch `.github/workflows/prepare-release.yml` with that tag. It runs core
   verification and `.github/workflows/distribution.yml`, then creates a draft
   with the release artifacts. Wait for both prerequisites and draft creation.
6. Check the draft's tag, commit, notes, archive versions, licenses, and canonical
   GitHub download URLs. Verify both native installer scripts and their checksums
   are attached. Publish only the workflow's distribution artifacts; smoke-test
   download URLs and installed binaries must not be published.

## Publish

Publish the verified draft through the GitHub UI or a user-authenticated `gh`
session. A release created with Actions' built-in `GITHUB_TOKEN` does not trigger
the downstream release workflow.

`.github/workflows/release.yml` handles the `released` event, verifies the tag,
then deploys through the `cargo`, `brew`, `npm`, and `registry` environments:

- Cargo and npm use trusted publishers matching `budna-marketplace/mcp`,
  `release.yml`, and their respective environment names.
- Brew uses `HOMEBREW_TOKEN` in `brew` to update
  `budna-marketplace/homebrew-tap`.
- After npm succeeds, `registry` publishes `server.json` using GitHub OIDC.
  The canonical `budna-mcp.tgz` contains the Registry's `mcpName` marker;
  cargo-dist's separate npm archive and its checksums remain unchanged.

Respect environment approvals. For failures, inspect the failed job and current
registry state before retrying failed jobs. Do not unpublish versions, replace
assets, or change protections to make a retry pass. Fix authentication settings
without rebuilding; source or workflow changes need a fresh version and tag.

## Verify

Confirm all deployment jobs succeed and independently check the released
version on each workspace crate, the tap formula and its install CI, and
the npm package's `latest` tag and official MCP Registry entry. Exercise the
installed CLI with `--version`, including the Shell and PowerShell installers.
Report partial publication explicitly; marketplace API calls are not required.
