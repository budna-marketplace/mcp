# Marketplace Explorer

- Extend the existing Budna design in `src/style.css`; reuse the bundled fonts
  and wordmark in `src/assets` and preserve host theme support.
- Preserve keyboard access, visible focus, accessible names, reduced motion,
  and responsive layouts. Keep user-facing strings in `src/i18n.ts`.
- Use `src/bridge.ts` for tool calls, links, and comparison requests. Render
  marketplace text with `textContent`; keep the app free of direct network
  requests, browser storage, and external scripts or stylesheets.

Run from this directory after UI changes:

```text
npm ci
npm audit --audit-level=high
npm run check
npm run build
```

Commit the regenerated `crates/budna-mcp-server/assets/marketplace-explorer-v1.html`
from the repository root; never edit that generated file by hand. The build runs
`scripts/check-asset.mjs` to enforce the bundle limit, allowed content, version
alignment, and dependency notices. Update
`crates/budna-mcp-server/assets/THIRD_PARTY_NOTICES.txt` when dependencies change.
