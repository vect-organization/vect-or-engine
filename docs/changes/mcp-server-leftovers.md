# engine:mcp-server-leftovers

Change-log row: S9 (`docs/changes/v0.4.0.csv`). This repository has no ledger file, so the entry lives here.
Status: closed 2026-11-06 (owner decision). The server code was removed; this note and the design documents are kept for a future rebuild as a proper MCP implementation (the removed server answered only its own JSON-RPC methods, not `initialize` or `tools/list`).

Surface: `mcp/server.mjs` (experimental HTTP server, `npm run start-mcp`).

## Leftovers

- CORS value: `Access-Control-Allow-Origin` is the bare string `127.0.0.1`. A valid value is a full origin (`http://127.0.0.1:4000`) or `null`; browsers ignore the bare host, so the header does nothing
- Token off by default: `MCP_AUTH_TOKEN` is opt-in. With it unset every request is accepted and the server only prints a warning; the Host / Origin gate is the only protection
- Stale binary names: `engineCandidates` lists `vect-or-engine-v0.3.0.node` and `vect-or-engine-napi.win32-x64-msvc.node`; current builds are named by the napi package scripts, so only the `index.js` fallback can match
- `OPTIONS` pre-flight returns 204 without the Host / Origin / token gate

## Decision needed (owner)

- Keep the server experimental and document the above, or remove it from the published package
- If kept: default the token to required (generate one at start), emit a real origin, drop the stale names

## Check

- Review only. Closing this entry needs one of the two decisions above and a matching change
