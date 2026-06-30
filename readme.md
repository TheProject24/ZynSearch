# ZynSearch

> A fast, multi-surface search engine with a Rust core, secure HTTP REST, gRPC, and thin SDKs for JavaScript, Python, and Go.

ZynSearch is built for the moment when a search engine should stop feeling like a research prototype and start feeling like a product.
It gives you a compact engine core, an authenticated HTTP layer for SDKs, a gRPC contract for strongly typed integrations, and a clean
path for embedding search directly inside Rust applications.

The project now spans the full stack:

- `zynsearch-core` for indexing, retrieval, storage, scoring, and query execution
- `zynsearch-server` for HTTP and gRPC delivery
- `zynsearch-cli` for local workflows and quick indexing/searching
- `sdks/zynsearch-js` for TypeScript and Node.js consumers
- `sdks/zynsearch-py` for Python applications and automation
- `sdks/zynsearch-go` for Go services and tooling

## Why ZynSearch Exists

Most search demos stop at "it works." ZynSearch is aimed at the next layer:

- a reusable search engine core
- a secure transport boundary for SDKs
- predictable JSON and gRPC contracts
- enough structure to grow into a real platform

It is especially useful when you want:

- a lightweight embedded search engine
- a self-hosted search service with auth
- SDKs that can talk to the same backend from multiple languages
- a codebase that makes the moving parts easy to understand

## Visual Map

```text
Documents -> Analyzer -> Inverted Index -> Scoring -> HTTP/gRPC -> SDKs
                                |
                                +-> Persistence / WAL / segments / query pipeline
```

## Core Components

### `zynsearch-core`

The engine room.

It contains:

- document ingestion and parsing
- tokenization and analysis
- inverted index structures
- query execution and ranking
- storage and persistence helpers
- the embeddable `ZynSearch` API

### `zynsearch-server`

The network boundary.

It exposes:

- `POST /index`
- `GET /search`
- `DELETE /index/:id`
- gRPC service methods for richer typed clients

It also supports:

- Basic Auth for SDK access
- optional TLS so credentials are not sent in cleartext

### SDKs

The SDKs are deliberately thin.

They translate native language objects into the REST contract, then return structured results back to the caller with as little ceremony as possible.

- JavaScript/TypeScript: ergonomic async client for Node.js and serverless apps
- Python: friendly sync and async access for data tooling and automation
- Go: small, direct client for backend services

## Security Model

For production use, the recommended shape is:

1. HTTPS enabled with certificates
2. Basic Auth enabled for client credentials
3. SDKs configured with endpoint, username, and password

That gives you a simple connection story without inventing custom auth schemes early.

## Project Highlights

- Rust 2024 core with a modular architecture
- REST and gRPC distribution layers
- SDK-friendly JSON payloads
- structured HTTP errors
- support for both embedded and service-based deployment
- a clear path toward production hardening

## Getting Started

The fastest way to understand ZynSearch is to start at the center:

- read [`crates/zynsearch-core/README.md`](/home/knny/Dev-Life/ZynSearch/crates/zynsearch-core/README.md)
- inspect [`crates/zynsearch-server/src/http.rs`](/home/knny/Dev-Life/ZynSearch/crates/zynsearch-server/src/http.rs)
- review [`proto/README.md`](/home/knny/Dev-Life/ZynSearch/proto/README.md)
- explore the SDK README files in `sdks/`

## Repository Layout

```text
.
├── crates/
│   ├── zynsearch-core/
│   ├── zynsearch-server/
│   └── zynsearch-cli/
├── sdks/
│   ├── zynsearch-go/
│   ├── zynsearch-js/
│   └── zynsearch-py/
├── proto/
└── docs/
```

## Design Philosophy

ZynSearch is trying to be three things at once:

- simple enough to learn from
- expressive enough to build on
- practical enough to ship

That means the repository intentionally keeps the engine core and the transport layers separate.
The result is a cleaner mental model and a better SDK story.

## Next Steps

If you are exploring the repo for the first time, a good path is:

1. read the core README
2. skim the protocol README
3. open the server HTTP implementation
4. inspect one SDK client

