<div align="center">

<img src="./assets/zynsearch-banner.png" alt="ZynSearch" width="100%">

<br>

![engine](https://img.shields.io/badge/engine-rust-bd5b34?style=flat-square&labelColor=241b16)
![index](https://img.shields.io/badge/index-inverted-c89a5b?style=flat-square&labelColor=241b16)
![ranking](https://img.shields.io/badge/ranking-BM25-bd5b34?style=flat-square&labelColor=241b16)
![transports](https://img.shields.io/badge/transports-REST%20%7C%20gRPC-c89a5b?style=flat-square&labelColor=241b16)
![sdks](https://img.shields.io/badge/SDKs-JS%20·%20Python%20·%20Go-7faa78?style=flat-square&labelColor=241b16)

</div>

<br>

> *A Rust search platform built to feel fast, elegant, and dependable across embedded, HTTP, gRPC, and SDK surfaces.*

ZynSearch is the kind of project that wants to be understood from the inside out. On the surface it's a search engine with a secure REST API, a typed gRPC service, and thin language SDKs. Underneath, it's an intentionally layered system — the core engine, the ranking math, the transport layers, and the SDK contracts are all kept separate, so each part can evolve without dragging the others down.

This README is the high-level technical map of the project. It's written to show the brain behind the code:

- 🧬 how documents become tokens
- 🗂️ how tokens become postings
- 📐 how postings become ranked results
- 🔐 how the server protects and exposes the engine
- 🧭 how each file in the repository fits into the larger machine

<br>

## 🎯 What ZynSearch Is Optimizing For

ZynSearch isn't trying to be a generic search demo. Search systems tend to become opaque very quickly — ZynSearch goes the other way, keeping its internal logic readable enough to be **audited, extended, and improved.**

| | Target | Why it matters |
|---|---|---|
| 🟤 | **Fast enough to feel responsive** | Hash-driven postings and bounded top-K avoid wasted work |
| 🟠 | **Clear enough to reason about** | Every transformation stays inspectable, not buried in handlers |
| 🟢 | **Secure enough to serve SDKs** | Basic Auth, optional TLS, structured errors at the boundary |
| 🟣 | **Modular enough to embed or deploy** | The same core runs headless or behind a full server |
| 🔶 | **Explicit about its own math** | IDF and TF are named, visible, and tunable — never a black box |

<br>

## 🧭 The Big Picture

The REST API, gRPC service, CLI, and SDKs are all different windows into the **same search brain.** Every document and every query passes through this same shape:

```text
document input
   → parsing and normalization
   → tokenization and stop-word filtering
   → inverted index insertion
   → query token analysis
   → document intersection / shard coordination
   → BM25 scoring and top-K ranking
   → transport serialization through HTTP or gRPC
```

<br>

## 🏛️ Core Engine Philosophy

The core engine lives in `crates/zynsearch-core`, built around four rules:

> 1. **Keep ingestion deterministic.**
> 2. **Keep search execution explainable.**
> 3. **Keep ranking math explicit.**
> 4. **Keep transport concerns out of the engine.**

That separation is what lets ZynSearch behave like a product instead of a pile of handlers.

<br>

## 📇 File Map — Who Does What

*🟤 core engine&nbsp;&nbsp;&nbsp;&nbsp;🟠 server&nbsp;&nbsp;&nbsp;&nbsp;🟢 proto contract*

| | File | Owns |
|---|---|---|
| 🟤 | [`engine.rs`](crates/zynsearch-core/src/engine.rs) | Document ingestion, search execution, shard-aware paths, deletion/cleanup, corpus ingestion. **First stop** for a document's lifecycle from arrival to removal. |
| 🟤 | [`index.rs`](crates/zynsearch-core/src/index.rs) | The inverted index and document registry — terms ↔ postings, document IDs ↔ source paths, metadata. |
| 🟤 | [`analyzer.rs`](crates/zynsearch-core/src/analyzer.rs) | Text normalization and token generation: lowercasing, splitting, stop-word filtering, light stemming. Decides what "same word" means. |
| 🟤 | [`searcher.rs`](crates/zynsearch-core/src/searcher.rs) | Search execution and ranking entry point — boolean intersection *and* BM25-scored retrieval. |
| 🟤 | [`bm25.rs`](crates/zynsearch-core/src/bm25.rs) | The ranking math: rare terms matter more, repetition saturates, long documents don't win by default. |
| 🟤 | [`query_pipeline.rs`](crates/zynsearch-core/src/query_pipeline.rs) | Query coordination — parses payloads, distributes across shards, merges & formats results. |
| 🟤 | [`top_k.rs`](crates/zynsearch-core/src/top_k.rs) | `TopKCollector` — a bounded heap that keeps the strongest `K` results without sorting the universe of matches. |
| 🟤 | [`storage.rs`](crates/zynsearch-core/src/storage.rs) | Persistence and restoration — serializes the index so the engine restarts without losing its corpus. |
| 🟠 | [`http.rs`](crates/zynsearch-server/src/http.rs) | The REST gateway — HTTP ↔ engine actions, Basic Auth, optional TLS, structured JSON errors. |
| 🟠 | [`main.rs`](crates/zynsearch-server/src/main.rs) | Server boot — which transport starts, HTTP/gRPC/TCP/both, auth & TLS settings, engine hydration. |
| 🟣 | [`sdks/*`](sdks) | Thin clients in JS/TS, Python, and Go. Intelligence stays in the core and transport layer — never duplicated in the wrapper. |

<br>

## 📐 How The Search Math Works

This is the heart of ZynSearch.

```text
score(q, d) = Σ  IDF(t) · TF(t, d)      for each term t in query q
```

<details>
<summary><b>1 · Term Frequency</b> — repetition helps, but with diminishing returns</summary>

<br>

```text
TF(t, d) = (f · (k1 + 1)) / (f + k1 · (1 − b + b · |d| / avgdl))
```

If a term appears more often in a document, that document usually scores higher for it — but raw repetition shouldn't dominate forever, so the score follows a saturation curve as `f` grows.

</details>

<details>
<summary><b>2 · Inverse Document Frequency</b> — rare terms are more discriminative</summary>

<br>

```text
IDF(t) = ln(1 + (N − n_t + 0.5) / (n_t + 0.5))
```

If almost every document contains a term, it should matter less than a word that appears in only a few. The numerator grows when a term is rare; the denominator grows when it's common.

</details>

<details>
<summary><b>3 · Length Normalization</b> — long documents shouldn't win just by being long</summary>

<br>

Long documents naturally contain more words, giving them more chances to match a query by accident. The `b` parameter and average document length (`avgdl`) compensate for that, avoiding a common search failure mode.

</details>

<details>
<summary><b>4 · Final Score</b> — combining evidence across query terms</summary>

<br>

```text
score(d, q) = Σ score(t, d)   for t in q
```

This lets ZynSearch combine evidence from multiple query terms instead of treating them independently.

</details>

**Variables, spelled out:**

| Symbol | Meaning |
|---|---|
| `N` | Total number of documents |
| `n_t` | Number of documents containing term `t` |
| `f` | Term frequency in document `d` |
| `\|d\|` | Document length |
| `avgdl` | Average document length across the corpus |
| `k1` | Controls saturation strength |
| `b` | Controls length normalization |

<br>

## 🔍 How The Retrieval Logic Works

The retrieval pipeline runs in two modes from the same index:

<table>
<tr>
<td width="50%" valign="top">

### Boolean intersection

Simple, fast, easy to explain.

1. Analyze the query into tokens
2. Look up each token in the inverted index
3. Build a candidate document-ID set per token
4. **Intersect** the sets
5. Map surviving IDs back to source paths

```text
matches(query) =
  documents(term₁) ∩ documents(term₂)
  ∩ … ∩ documents(termₙ)
```

If any term is missing, the result is empty immediately — an early exit that prevents wasted work.

</td>
<td width="50%" valign="top">

### Scored retrieval

More expressive — *how strongly* does it match?

1. Analyze the query
2. Collect candidate postings
3. Compute IDF per term
4. Compute BM25 contributions per posting
5. Sum scores per document
6. Sort the top candidates

```text
score(d, q) =
  Σ score(t, d)
  for t in q
```

Not just "does it match," but how strongly — across multiple terms at once.

</td>
</tr>
</table>

<br>

## 🧩 Sharding & Coordination

ZynSearch supports shard-aware execution — split the work, search each partition independently, merge at the coordinator. `query_pipeline.rs` and `searcher.rs` collaborate on this; the current implementation shards by **document ID modulo shard count**, so each document is deterministically routed.

| | Why it matters |
|---|---|
| ⚙️ | Keeps search **parallelizable** |
| 🧱 | Prevents the coordinator from becoming the **only bottleneck** |
| 📈 | Makes the system **easier to scale horizontally** later |

<br>

## 🗃️ Why The Inverted Index Is The Right Shape

Search engines don't search by scanning every document — too slow. Instead, they flip the question:

<table>
<tr>
<td width="50%" valign="top">

**Forward index** — *wrong question*
> "what terms are in this document?"

```text
doc_42 → [rust, async, index, http]
doc_43 → [token, stem, parser, http]
```

</td>
<td width="50%" valign="top">

**Inverted index** — *right question*
> "what documents contain this term?"

```text
http  → [doc_42, doc_43]
token → [doc_43]
```

</td>
</tr>
</table>

Search cares about the second question — so ZynSearch stores term-centric posting lists instead of document-centric text blobs.

<br>

## ⏱️ Complexity Intuition

The exact cost depends on corpus and query, but the shapes are easy to describe:

| Operation | Shape |
|---|---|
| Tokenization | Linear in input size |
| Postings lookup | Hash-map driven — effectively constant time per term |
| Boolean intersection | Depends on posting list sizes |
| BM25 ranking | Proportional to candidate postings actually inspected |
| Top-K collection | Avoids a full sort of every match |

The practical lesson: do the least work necessary to answer the query well.

<br>

## 🗂️ File-By-File Responsibility Summary

<table>
<tr><td>

**🟤 Ingestion & parsing**
- `parser.rs` — raw file content → cleaner search text
- `crawler.rs` — finds candidate files
- `document_ingest.rs` / `ingestion.rs` — corpus loading paths

</td><td>

**🟤 Core index & engine**
- `index.rs` — terms, documents, metadata
- `engine.rs` — mutates & queries the index
- `storage.rs` — serializes state

</td></tr>
<tr><td>

**🟤 Search & ranking**
- `analyzer.rs` — normalizes text
- `searcher.rs` — match logic & scoring
- `bm25.rs` — ranking weights
- `top_k.rs` — strongest results
- `query_pipeline.rs` — distributed execution & formatting

</td><td>

**🟠 Server & APIs**
- `zynsearch-server/src/http.rs` — serves REST
- `zynsearch-server/src/main.rs` — boots transports & config
- `proto/zynsearch/v1/zynsearch.proto` — defines the gRPC contract

</td></tr>
<tr><td colspan="2">

**🟣 SDK surfaces**
- `sdks/zynsearch-js` → JS & TS &nbsp;|&nbsp; `sdks/zynsearch-py` → Python &nbsp;|&nbsp; `sdks/zynsearch-go` → Go

</td></tr>
</table>

<br>

## 🔐 Security & Delivery Model

The recommended production stack:

> 1. **HTTPS** with certificates
> 2. **Basic Auth** for request identity
> 3. **REST or gRPC**, depending on integration style
> 4. **SDKs that talk to the server** — never to the core directly

That model keeps the engine secure while keeping integration simple.

<br>

## 🧱 Why The Separation Matters

The split between core, server, and SDKs gives ZynSearch real advantages:

- ✅ The engine can be reused **without networking**
- ✅ The server can evolve **without touching scoring math**
- ✅ SDKs stay idiomatic **without duplicating engine logic**
- ✅ The repository stays explainable **as it grows**

That separation is a major part of the project's identity.

<br>

## 🧭 Where To Go Next

If you want to understand ZynSearch properly, read it in this order:

1. [`crates/zynsearch-core/README_ZynSearch_Core.md`](crates/zynsearch-core/README_ZynSearch_Core.md)
2. [`crates/zynsearch-server/src/http.rs`](crates/zynsearch-server/src/http.rs)
3. [`proto/README_ZynSearch_Proto.md`](proto/README_ZynSearch_Proto.md)
4. [`sdks/zynsearch-js/README_ZynSearch_JavaScript_SDK.md`](sdks/zynsearch-js/README_ZynSearch_JavaScript_SDK.md)
5. [`sdks/zynsearch-py/README_ZynSearch_Python_SDK.md`](sdks/zynsearch-py/README_ZynSearch_Python_SDK.md)
6. [`sdks/zynsearch-go/README_ZynSearch_Go_SDK.md`](sdks/zynsearch-go/README_ZynSearch_Go_SDK.md)

<br>

<div align="center">

*ZynSearch — core, server, and SDKs, kept honest about how they fit together.*

</div>
