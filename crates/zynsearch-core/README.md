# zynsearch-core

`zynsearch-core` is the engine heart of ZynSearch: the crate you embed when you want search without the network.

It is responsible for the full inner loop:

- ingesting documents
- analyzing text into tokens
- building and querying the inverted index
- scoring results
- persisting and restoring index state

## What Lives Here

This crate contains the parts that make ZynSearch feel like a search engine rather than just a router:

- `crawler` for filesystem discovery
- `parser` for lightweight document cleanup
- `analyzer` for token normalization
- `index` for postings and registries
- `searcher` for query execution
- `engine` for ingestion and search orchestration
- `storage` for persistence
- `query_pipeline` for higher-level query flow

## Why It Matters

The separation is intentional.

`zynsearch-core` can be used directly by Rust applications that want:

- no server process
- no HTTP overhead
- direct control over indexing and retrieval
- a compact, embeddable search backend

That makes it a good fit for:

- local desktop tools
- command-line utilities
- internal search features
- backend services that want search in-process

## Core Shape

```text
text -> analyzer -> tokens -> inverted index -> scoring -> results
```

The crate is designed to keep that path easy to follow and easy to extend.

## Public API

The primary embedded interface is `ZynSearch`.

It gives you:

- `index_text()`
- `index_document()`
- `index_tokens()`
- `search()`
- `search_scored()`
- `delete_document()`
- `save()` and `load()`

## Developer Experience

The goal is not just correctness.
It is clarity.

You should be able to read through the crate and understand:

- how a document becomes searchable
- how query terms are matched
- how the index is persisted
- how higher-level transports sit on top of the engine

## Relationship To The Rest Of ZynSearch

`zynsearch-core` powers:

- the REST server
- the gRPC server
- the CLI
- the SDKs

If the project is the city, this crate is the power plant.

