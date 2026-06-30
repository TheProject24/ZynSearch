# ZynSearch

> A Rust search platform built to feel fast, elegant, and dependable across embedded, HTTP, gRPC, and SDK surfaces.

ZynSearch is the kind of project that wants to be understood from the inside out.
On the surface it is a search engine with a secure REST API, typed gRPC service, and thin language SDKs.
Underneath, it is an intentionally layered system where the core engine, ranking math, transport layers, and SDK contracts are all separated so each part can evolve without dragging the others down.

This README is the high-level technical map of the project.
It is written to show the brain behind the code:

- how documents become tokens
- how tokens become postings
- how postings become ranked results
- how the server protects and exposes the engine
- how each file in the repository fits into the larger machine

## What ZynSearch Is Optimizing For

ZynSearch is not trying to be a generic search demo.
It is trying to be a real engine shape:

- fast enough to feel responsive
- clear enough to reason about
- secure enough to serve SDKs
- modular enough to embed or deploy
- explicit enough that the math and data flow stay visible

That design target matters because search systems tend to become opaque very quickly.
ZynSearch goes the other way: it keeps the internal logic readable so the system can be audited, extended, and improved.

## The Big Picture

```text
document input
   -> parsing and normalization
   -> tokenization and stop-word filtering
   -> inverted index insertion
   -> query token analysis
   -> document intersection / shard coordination
   -> BM25 scoring and top-K ranking
   -> transport serialization through HTTP or gRPC
```

The important thing is that the same core engine sits underneath all transports.
The REST API, gRPC service, CLI, and SDKs are all different windows into the same search brain.

## Core Engine Philosophy

The core engine lives in `crates/zynsearch-core`.
It is built around four ideas:

1. Keep ingestion deterministic.
2. Keep search execution explainable.
3. Keep ranking math explicit.
4. Keep transport concerns out of the engine.

That separation is what lets ZynSearch behave like a product instead of a pile of handlers.

## File Map: Who Does What

### `crates/zynsearch-core/src/engine.rs`

The orchestration layer for the engine.

This file owns:

- document ingestion into the index
- search execution against the in-memory index
- shard-aware execution paths
- deletion and cleanup logic
- corpus directory ingestion

If you want to understand the lifecycle of a document from arrival to removal, this file is the first stop.

### `crates/zynsearch-core/src/index.rs`

The inverted index and document registry.

This is where ZynSearch stores the relationship between:

- terms and postings
- document IDs and source paths
- metadata and source kinds

Conceptually, the index is a term-to-document map:

```text
term -> [(doc_id, frequency), (doc_id, frequency), ...]
```

This structure is what makes search efficient.
Instead of scanning every document, the engine jumps directly to the term buckets relevant to the query.

### `crates/zynsearch-core/src/analyzer.rs`

Text normalization and token generation.

The analyzer handles the first mathematical transformation in the system:

raw text -> normalized token sequence

This usually means:

- lowercasing
- splitting on punctuation and whitespace
- filtering stop words
- applying lightweight stemming rules

The analyzer decides what "same word" means inside the engine.
That decision has a huge impact on recall, precision, and ranking stability.

### `crates/zynsearch-core/src/searcher.rs`

Search execution and ranking entry point.

This file performs two major kinds of retrieval:

- boolean-style intersection search
- scored retrieval using BM25

It is the main consumer of the inverted index and the analyzer.

### `crates/zynsearch-core/src/bm25.rs`

The ranking math.

This file encodes the probabilistic intuition behind the results:

- rare terms should matter more than common terms
- repeated occurrences inside a document should help, but with diminishing returns
- long documents should not win simply because they are long

BM25 captures those ideas in a compact formula:

```text
score(q, d) = sum over terms t in q of IDF(t) * TF(t, d)
```

In ZynSearch the shape is closer to:

```text
IDF(t) = ln(1 + (N - n_t + 0.5) / (n_t + 0.5))

TF(t, d) = (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d| / avgdl))
```

Where:

- `N` is the total number of documents
- `n_t` is the number of documents containing term `t`
- `f` is term frequency in document `d`
- `|d|` is the document length
- `avgdl` is average document length
- `k1` controls saturation strength
- `b` controls length normalization

This gives ZynSearch a stable, explainable ranking model.

### `crates/zynsearch-core/src/query_pipeline.rs`

The query coordination layer.

This file is responsible for:

- parsing incoming query payloads
- distributing search across shards
- merging and formatting results
- selecting output representations

The query coordinator is where search stops being just "find documents" and becomes "find the best documents, across shards, in one consistent response."

### `crates/zynsearch-core/src/top_k.rs`

Result capping and priority handling.

Search engines should not sort everything if they only need the best few hits.
`TopKCollector` keeps the highest scoring results in a bounded heap so the engine can preserve the winners without doing unnecessary global work.

That gives the system a more efficient result selection strategy:

- insert candidates
- retain only the strongest `K`
- sort the final small set

This is a classic engineering tradeoff: a small heap gives you a better asymptotic shape than sorting the full universe of matches.

### `crates/zynsearch-core/src/storage.rs`

Persistence and restoration.

This file is the bridge between volatile memory and durable state.
It serializes the index so the engine can restart without losing the current corpus.

The format is designed to preserve:

- document registry entries
- postings
- metadata
- enough structure to restore the search state quickly

### `crates/zynsearch-server/src/http.rs`

The REST gateway.

This file translates HTTP requests into engine actions and engine actions back into JSON.
It also enforces:

- Basic Auth
- optional TLS
- structured error responses
- SDK-friendly request and response shapes

### `crates/zynsearch-server/src/main.rs`

Server boot and transport selection.

This file decides:

- which transport starts
- whether the server runs HTTP, gRPC, TCP, or both
- which auth and TLS settings apply
- how the engine is hydrated on boot

### `sdks/*`

Thin clients.

Each SDK exists to make the same backend feel natural in its own language.
The SDKs should stay small because the intelligence belongs in the core and the transport layer, not in the client wrappers.

## How The Search Math Works

This is the heart of ZynSearch.

### 1. Term Frequency

If a term appears more often in a document, that document should usually score higher for that term.
But raw repetition should not dominate forever, so the score uses a saturation curve.

That is what the `TF` side of BM25 does:

```text
TF = (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d| / avgdl))
```

Where:

- `f` is the term frequency
- `|d| / avgdl` is a document length ratio

As `f` grows, the score increases but with diminishing returns.

### 2. Inverse Document Frequency

Rare terms are more discriminative.
If almost every document contains a term, it should matter less than a word that appears in only a few documents.

That is what IDF models:

```text
IDF = ln(1 + (N - n_t + 0.5) / (n_t + 0.5))
```

The numerator grows when the term is rare.
The denominator grows when the term is common.

### 3. Length Normalization

Long documents naturally contain more words, which means they have more opportunities to match a query by chance.
BM25 compensates for that with the `b` parameter and average document length.

That helps avoid a common search failure mode where long documents win purely because they are long.

### 4. Final Score

The final score is the sum of per-term contributions.

```text
score(d, q) = sum(score(t, d)) for t in q
```

This lets ZynSearch combine evidence from multiple query terms instead of treating them independently.

## How The Retrieval Logic Works

The retrieval pipeline has two modes in the core:

### Boolean intersection search

For basic search, the engine:

1. analyzes the query into tokens
2. looks up each token in the inverted index
3. builds a set of candidate document IDs per token
4. intersects the sets
5. maps surviving document IDs back to document paths

This is simple, fast, and easy to explain.

The logic is essentially:

```text
matches(query) = documents(term1) ∩ documents(term2) ∩ ... ∩ documents(termN)
```

If any term is missing, the result is empty immediately.
That early exit is a performance win because it prevents wasted work.

### Scored retrieval

For ranked search, the engine:

1. analyzes the query
2. collects candidate postings
3. computes IDF per term
4. computes BM25 contributions per posting
5. sums scores per document
6. sorts the top candidates

This is the more expressive mode because it gives the engine a way to say not only "does it match?" but also "how strongly does it match?"

## Sharding And Coordination

ZynSearch supports shard-aware execution.

The reasoning is straightforward:

- split work across partitions
- search each partition independently
- merge the results at the coordinator

This is what `query_pipeline.rs` and `searcher.rs` collaborate on.

The shard function uses the document ID modulo the shard count in the current implementation.
That means a document is deterministically assigned to one shard and can be revisited by the same routing rule during search.

This matters because:

- it keeps search parallelizable
- it prevents the coordinator from becoming the only bottleneck
- it makes the system easier to scale horizontally later

## Why The Inverted Index Is The Right Shape

Search engines do not usually search by scanning every document.
That would be too slow.

Instead, they use an inverted index:

```text
term -> list of documents containing the term
```

The difference is huge:

- forward indexing answers "what terms are in this document?"
- inverted indexing answers "what documents contain this term?"

Search cares about the second question.
That is why ZynSearch stores term-centric posting lists instead of document-centric raw text blobs.

## Complexity Intuition

The exact cost depends on the corpus and query, but the main shapes are easy to describe:

- tokenization is linear in input size
- postings lookup is hash-map driven and effectively constant-time per term lookup
- boolean intersection depends on posting list sizes
- BM25 ranking is proportional to the number of candidate postings inspected
- top-K collection avoids a full sort of all documents

The practical lesson is that the engine tries to do the least amount of work necessary to answer the query well.

## File-by-File Responsibility Summary

### Ingestion and parsing

- `parser.rs` turns raw file content into cleaner search text
- `crawler.rs` finds candidate files
- `document_ingest.rs` and `ingestion.rs` manage corpus loading paths

### Core index and engine

- `index.rs` stores terms, documents, and metadata
- `engine.rs` mutates and queries the index
- `storage.rs` serializes state

### Search and ranking

- `analyzer.rs` normalizes text
- `searcher.rs` performs match logic and scoring
- `bm25.rs` computes ranking weights
- `top_k.rs` selects the strongest results
- `query_pipeline.rs` coordinates distributed execution and formatting

### Server and APIs

- `crates/zynsearch-server/src/http.rs` serves REST
- `crates/zynsearch-server/src/main.rs` boots transports and config
- `proto/zynsearch/v1/zynsearch.proto` defines the gRPC contract

### SDK surfaces

- `sdks/zynsearch-js` targets JS and TS
- `sdks/zynsearch-py` targets Python
- `sdks/zynsearch-go` targets Go

## Security And Delivery Model

The recommended production stack is:

1. HTTPS with certificates
2. Basic Auth for request identity
3. REST or gRPC depending on the integration style
4. SDKs that talk to the server, not the core directly

That model keeps the engine secure while still making the integration experience simple.

## Why The Separation Matters

The split between core, server, and SDKs gives ZynSearch several advantages:

- the engine can be reused without networking
- the server can evolve without touching scoring math
- SDKs can be idiomatic without duplicating engine logic
- the repository stays explainable as it grows

That separation is a major part of the project identity.

## Where To Go Next

If you want to understand ZynSearch properly, read it in this order:

1. [`crates/zynsearch-core/README_ZynSearch_Core.md`](/home/knny/Dev-Life/ZynSearch/crates/zynsearch-core/README_ZynSearch_Core.md)
2. [`crates/zynsearch-server/src/http.rs`](/home/knny/Dev-Life/ZynSearch/crates/zynsearch-server/src/http.rs)
3. [`proto/README_ZynSearch_Proto.md`](/home/knny/Dev-Life/ZynSearch/proto/README_ZynSearch_Proto.md)
4. [`sdks/zynsearch-js/README_ZynSearch_JavaScript_SDK.md`](/home/knny/Dev-Life/ZynSearch/sdks/zynsearch-js/README_ZynSearch_JavaScript_SDK.md)
5. [`sdks/zynsearch-py/README_ZynSearch_Python_SDK.md`](/home/knny/Dev-Life/ZynSearch/sdks/zynsearch-py/README_ZynSearch_Python_SDK.md)
6. [`sdks/zynsearch-go/README_ZynSearch_Go_SDK.md`](/home/knny/Dev-Life/ZynSearch/sdks/zynsearch-go/README_ZynSearch_Go_SDK.md)

