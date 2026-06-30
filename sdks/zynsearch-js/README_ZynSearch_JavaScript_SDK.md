# zynsearch-js

`zynsearch-js` is the JavaScript and TypeScript client for ZynSearch.
It is meant to feel natural in modern Node.js applications while staying close to the REST contract.

## Highlights

- async-first API
- Basic Auth support
- request timeout configuration
- camelCase client methods
- JSON-native request and response handling

## API Surface

- `index(...)`
- `search(...)`
- `delete(...)`

## Why It Fits

JavaScript teams typically want:

- quick setup
- readable request code
- predictable errors
- little transport noise

This client is built for exactly that.

## Connection Model

Recommended production use:

- HTTPS endpoint
- username and password
- request timeout
- structured JSON errors from the server

## Best Fit

This SDK works well for:

- Node.js services
- serverless functions
- browser-adjacent backends
- TypeScript application layers
