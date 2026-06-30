# ZynSearch Proto

This directory contains the canonical gRPC contract for ZynSearch.
It is the source of truth for typed clients across languages.

If the REST layer is the friendly face of the service, the proto definition is the
high-precision handshake underneath it.

## What’s Inside

```text
proto/
├── buf.yaml
├── buf.gen.yaml
└── zynsearch/
    └── v1/
        └── zynsearch.proto
```

## Service Overview

| RPC | Style | Purpose |
| --- | --- | --- |
| `Index` | unary | Index one document |
| `Search` | unary | Retrieve ranked results |
| `Delete` | unary | Remove a document |
| `BulkIndex` | client streaming | Push many documents over one connection |
| `SearchStream` | server streaming | Receive results as they are scored |

## Why It Matters

The proto file gives ZynSearch:

- strong typing
- cross-language compatibility
- a clean contract for SDK generation
- a stable foundation for future transport evolution

## Recommended Tooling

Buf is the easiest path for generating and validating the schema.
It keeps linting, breaking-change checks, and code generation in one workflow.

## Relationship To The REST API

The gRPC contract and the HTTP API expose the same product idea through different doors:

- REST for SDKs, simple integrations, and web-friendly clients
- gRPC for typed services and higher-throughput machine-to-machine traffic
