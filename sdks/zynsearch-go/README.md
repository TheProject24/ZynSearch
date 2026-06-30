# zynsearch-go

`zynsearch-go` is the Go client for ZynSearch.
It wraps the REST API with a small, direct API that fits naturally into backend services and tooling.

## Highlights

- Basic Auth support
- simple `Index`, `Search`, and `Delete` methods
- JSON request/response handling
- endpoint-based configuration
- a clean fit for services that prefer Go over heavier SDK abstractions

## Design Philosophy

The Go client is intentionally small.
It should feel like a familiar HTTP wrapper, not a framework.

## Core Operations

- `Index`
- `Search`
- `Delete`

## Connection Model

Recommended production use:

- HTTPS endpoint
- username and password
- structured JSON errors

## Best Fit

This SDK is a good fit for:

- Go microservices
- internal tooling
- background workers
- API adapters that need a small dependency surface

