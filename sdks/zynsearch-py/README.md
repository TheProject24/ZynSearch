# zynsearch-py

`zynsearch-py` is the Python SDK for ZynSearch.
It speaks the REST API with a friendly Pythonic surface while keeping the contract close to the backend.

## What It Gives You

- synchronous and asynchronous client entry points
- Basic Auth support
- endpoint, timeout, and request configuration
- typed index/search/delete calls
- a lightweight path for data tools and automation

## Why It Exists

Python users usually want two things:

- fast integration
- minimal ceremony

This SDK is designed to keep the ergonomics simple while still matching the server contract closely.

## Typical Workflow

```text
create client -> authenticate -> index content -> search -> delete when needed
```

## API Shape

The client centers on three operations:

- `index(...)`
- `search(...)`
- `delete(...)`

## Connection Story

The SDK is built for the production pattern used by ZynSearch:

- HTTPS endpoint
- username and password
- structured JSON responses

That makes it straightforward to use from scripts, notebooks, and services.

## Best Fit

This SDK is a good fit for:

- automation jobs
- data pipelines
- internal search tooling
- Python backends that want a simple search client

