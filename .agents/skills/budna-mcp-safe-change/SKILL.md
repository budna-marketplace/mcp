---
name: budna-mcp-safe-change
description: Change Budna MCP tools, API models, capability policy, schemas, configuration, transports, or crate packaging while preserving the public capability and privacy boundary.
---

# Budna MCP Safe Change

Read root `AGENTS.md` for capability, publication, privacy, and verification
requirements. Paths below are relative to the repository root.

## Implement the change

1. Trace the change through client input, private wire decoding, public response
   model, capability policy, MCP parameters, output projection, and tool route.
2. Check callers, transports, and the Marketplace Explorer bridge before
   changing a tool or response shape.
3. Declare only response fields required by the public tool and build outputs
   from explicit projection structs. Keep visibility decisions inside the
   client boundary.
4. Validate IDs, text lengths, pages, limits, prices, filter names, filter values,
   and sort values at runtime.
5. Fail closed when a record or search result does not satisfy the public
   visibility policy. Return unavailable records indistinguishably.
6. Preserve the untrusted-content notice when changing projections or UI output.

## Prove the boundary

Use synthetic tests that verify:

- the complete allowed key set for every changed output;
- additional response fields are ignored rather than retained;
- non-public and unknown visibility states fail closed;
- hostile marketplace text remains ordinary returned data;
- invalid and unknown inputs produce structured tool errors;
- timeouts, retries, response-size limits, and concurrency bounds still hold.

## Verify before completion

Run root `AGENTS.md` verification, including its stdio flow when applicable.
For UI changes, also follow `ui/marketplace-explorer/AGENTS.md`.
Inspect `cargo package --package <crate> --list --locked` for each changed
publishable crate. For release preparation or publication, follow
`.agents/skills/budna-mcp-release/SKILL.md`.
