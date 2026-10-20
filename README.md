# GraphQL (DBX plugin)

GraphQL client for [DBX](https://dbxio.com).

## Features (v0.1)

- Connection form: endpoint URL, Bearer token, extra headers (JSON)
- Query editor (CodeMirror 6 + schema autocomplete)
- Response as collapsible JSON tree (`svelte-json-tree`)
- Run via Rust sidecar (no browser CORS)
- Altair-style schema Docs Explorer
- Query history and draft persistence (`host.storage`)

## Develop

Requires **Node.js 22.14.0** (see `.nvmrc`) and a Rust toolchain.

```bash
nvm use
npm install
npm run build
dbx-plugin dev --path . --port 5190
```

## Package

```bash
nvm use
npm run build
dbx-plugin package .
```
