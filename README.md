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

## 🧭 Configuration Reference

This section is the practical operating manual for ZynSearch.

If you only remember one thing, remember this:

> **Configuration is layered.**
> The effective runtime configuration is built in this order:
> 1. built-in defaults
> 2. `zynsearch.config.json` or `--config`
> 3. env file overrides via `--env-path` or `ZYN_ENV_PATH`
> 4. CLI flags and environment variables

That means the final value you observe at runtime may differ from what is written in the JSON file if you override it from the shell.

<br>

### 1 · How ZynSearch Resolves Configuration

ZynSearch reads config at startup, not at compile time.

The loader does the following:

1. Parse CLI arguments
2. Choose the config file path
3. Load the JSON config if it exists
4. Optionally load an env file
5. Apply CLI / env overrides
6. Normalize distribution channels
7. Derive the transport protocol when needed

That means a restart is enough for config changes to take effect.

<br>

### 2 · Config File Selection

The JSON config path is resolved in this order:

| Priority | Source | Example |
|---|---|---|
| 1 | `--config <path>` | `--config ./zynsearch.config.json` |
| 2 | `ZYN_CONFIG_PATH` | `export ZYN_CONFIG_PATH=/tmp/zynsearch.config.json` |
| 3 | Default | `zynsearch.config.json` in the current working directory |

If the file is missing, ZynSearch falls back to built-in defaults.

<br>

### 3 · CLI Flags Reference

The CLI is powered by `clap`, and each flag has a matching environment variable.

| Flag | Env var | Type | Meaning | Default / Notes |
|---|---|---|---|---|
| `--config` | `ZYN_CONFIG_PATH` | path | Path to the JSON config file | Defaults to `zynsearch.config.json` |
| `--env-path` | `ZYN_ENV_PATH` | path | Optional env file with `KEY=VALUE` overrides | If missing, no env-file overrides are applied |
| `-H, --host` | `ZYN_HOST` | string | Bind host for network transports | Usually `127.0.0.1` for local testing |
| `-P, --port` | `ZYN_PORT` | u16 | Bind port for network transports | Defaults to `7777` |
| `-D, --db-path` | `ZYN_DB_PATH` | path | Path to the serialized index database | Defaults to `index.bin` |
| `-C, --corpus-dir` | `ZYN_CORPUS_DIR` | path | Directory to ingest when using local folder ingestion | Overrides `ingestion.corpus_dir` |
| `-F, --output-format` | `ZYN_FORMAT` | `text` \| `json` \| `binary` | CLI output format and some transport responses | Defaults to `text` |
| `--protocol` | `ZYN_PROTOCOL` | `tcp` \| `http` \| `grpc` \| `both` | Which transport(s) to start | Derived from `distribution.channels` if not set |
| `--ingestion` | `ZYN_INGESTION` | `localdir` \| `s3` | Which ingestion source to use | Defaults to `localdir` |
| `--distribution` | `ZYN_DISTRIBUTION` | repeatable channel flag | Adds a client-facing transport channel | Repeat it to enable more than one |
| `--s3-bucket` | `ZYN_S3_BUCKET` | string | S3 bucket name for S3 ingestion | Required when `--ingestion s3` |
| `--s3-prefix` | `ZYN_S3_PREFIX` | string | Optional prefix within the bucket | Useful for corpus subfolders |
| `--query` | `ZYN_QUERY` | string | One-shot query on startup | Prints results and exits in CLI / short-circuit mode |
| `--http-username` | `ZYN_HTTP_USERNAME` | string | Basic Auth username for REST | Optional, but recommended for exposed servers |
| `--http-password` | `ZYN_HTTP_PASSWORD` | string | Basic Auth password for REST | Optional, but recommended for exposed servers |
| `--http-tls-cert-path` | `ZYN_HTTP_TLS_CERT_PATH` | path | TLS certificate chain for HTTPS | Must be paired with key path |
| `--http-tls-key-path` | `ZYN_HTTP_TLS_KEY_PATH` | path | TLS private key for HTTPS | Must be paired with cert path |
| `--enable-periodic-cleanup` | `ZYN_CLEANUP_ENABLE` | bool | Enable background cleanup of missing files | Defaults to `true` |
| `--cleanup-interval-seconds` | `ZYN_CLEANUP_INTERVAL` | u64 | Cleanup interval in seconds | Defaults to `60` |

<br>

### 4 · JSON Config Reference

The canonical JSON file is `zynsearch.config.json`.

It is made of six blocks:

| Block | Purpose |
|---|---|
| `manifest` | Human-readable identity for the deployment |
| `runtime` | Network behavior, query behavior, and output formatting |
| `ingestion` | Where documents come from |
| `storage` | Where the serialized index is stored |
| `distribution` | Which transport channels are exposed to clients |
| `cleanup` | Background maintenance of missing/deleted files |

<br>

#### 4.1 `manifest`

This block is descriptive metadata.

| Field | Type | Meaning | Default |
|---|---|---|---|
| `name` | string | Human name shown in startup banners and docs | `zynsearch` |
| `version` | string | Human version string | `1.0.0` |
| `description` | string | Short product description | `Plug-and-play search engine` |

Example:

```json
"manifest": {
  "name": "zynsearch",
  "version": "1.0.0",
  "description": "Plug-and-play search engine"
}
```

<br>

#### 4.2 `runtime`

This block controls how the process behaves when it starts serving or printing results.

| Field | Type | Meaning | Default | Notes |
|---|---|---|---|---|
| `host` | string | Interface the server binds to | `127.0.0.1` | Use `0.0.0.0` if you want LAN access |
| `port` | u16 | Port for TCP/HTTP/gRPC | `7777` | Must not conflict with other services |
| `protocol` | `tcp` \| `http` \| `grpc` \| `both` | Transport mode | `tcp` | CLI one-shot mode still loads this, but server startup uses it directly |
| `output_format` | `text` \| `json` \| `binary` | Result formatting style | `text` | Affects CLI and HTTP/gRPC formatting paths |
| `query` | string \| null | Optional one-shot query | `null` | If set, startup runs the query and exits instead of serving |
| `http_username` | string \| null | Basic Auth username | `null` | Enables auth when paired with password |
| `http_password` | string \| null | Basic Auth password | `null` | Should be treated like a secret |
| `http_tls_cert_path` | string \| null | PEM certificate path | `null` | Must be used with key path |
| `http_tls_key_path` | string \| null | PEM private key path | `null` | Must be used with cert path |

Example:

```json
"runtime": {
  "host": "127.0.0.1",
  "port": 7777,
  "protocol": "http",
  "output_format": "json",
  "query": null,
  "http_username": "admin",
  "http_password": "change-me",
  "http_tls_cert_path": null,
  "http_tls_key_path": null
}
```

Behavior notes:

- If `query` is set, the binary performs a one-shot query and exits after printing results.
- `protocol` determines which server transport starts.
- `output_format` matters most for CLI and for non-HTTP presentation paths.
- HTTP auth is only active when both `http_username` and `http_password` are present.
- HTTPS only starts when both TLS paths are present.

<br>

#### 4.3 `ingestion`

This block controls how documents are discovered and normalized into the index.

| Field | Type | Meaning | Default | Notes |
|---|---|---|---|---|
| `mode` | `localdir` \| `s3` | Ingestion source type | `localdir` | Local folder ingest is the easiest way to test |
| `corpus_dir` | string | Folder to crawl when `mode = localdir` | `./` | Use an absolute path for less surprise |
| `s3_bucket` | string \| null | S3 bucket name when `mode = s3` | `null` | Required for S3 ingest |
| `s3_prefix` | string \| null | Optional S3 prefix | `null` | Acts like a folder prefix |

Example:

```json
"ingestion": {
  "mode": "localdir",
  "corpus_dir": "/home/knny/itcannotalwaysbenight/MATERIALS",
  "s3_bucket": null,
  "s3_prefix": null
}
```

Behavior notes:

- `localdir` recursively crawls the folder and ingests supported files.
- Supported local extensions include: `.txt`, `.md`, `.csv`, `.pdf`, `.docx`, `.xlsx`.
- `s3` lists objects under the chosen prefix and ingests supported object types.
- If one document fails to extract, the current ingestion path skips it with a warning and continues.

<br>

#### 4.4 `storage`

This block decides where the serialized index is written.

| Field | Type | Meaning | Default | Notes |
|---|---|---|---|---|
| `db_path` | string | File path for the persisted index | `index.bin` | Should point to a writable location |

Example:

```json
"storage": {
  "db_path": "/home/knny/itcannotalwaysbenight/MATERIALS/zynsearch-index.bin"
}
```

Behavior notes:

- If the file exists, startup tries to load it first.
- If loading fails, ZynSearch falls back to re-ingesting the corpus.
- If you want a clean ingest test, remove the file before restarting.

<br>

#### 4.5 `distribution`

This block tells ZynSearch which client-facing transport channels should be available.

| Field | Type | Meaning | Default | Notes |
|---|---|---|---|---|
| `channels` | array of `tcp` \| `http` \| `grpc` | Enabled distribution channels | `[tcp]` | Duplicate values are deduplicated at startup |

Example:

```json
"distribution": {
  "channels": ["tcp", "http", "grpc"]
}
```

Behavior notes:

- A single channel maps to the matching protocol mode.
- Any combination of multiple channels maps to `both` at runtime.
- If the array is empty, ZynSearch normalizes it back to `[tcp]`.

<br>

#### 4.6 `cleanup`

This block controls background cleanup of documents whose underlying filesystem paths no longer exist.

| Field | Type | Meaning | Default | Notes |
|---|---|---|---|---|
| `enable_periodic_cleanup` | bool | Whether cleanup runs in the background | `true` | Useful for filesystem-backed corpora |
| `period_seconds` | u64 | Cleanup interval in seconds | `60` | Values lower than `1` are clamped by the runtime caller |

Example:

```json
"cleanup": {
  "enable_periodic_cleanup": true,
  "period_seconds": 60
}
```

Behavior notes:

- Useful when your corpus is a live folder that changes over time.
- Less relevant for immutable corpora or S3-backed corpora.

<br>

### 5 · Override Rules

When the same setting is provided more than once, the higher-priority source wins.

| Priority | Source | Wins Over |
|---|---|---|
| 1 | CLI flags | Everything below |
| 2 | `ZYN_*` environment variables | JSON file and defaults |
| 3 | env file values from `--env-path` | JSON file and defaults |
| 4 | `zynsearch.config.json` | Built-in defaults |
| 5 | Built-in defaults | Nothing |

That means a value in the JSON file is the baseline, not the ceiling.

<br>

### 6 · Practical Recipes

#### Recipe A: Local folder ingest + HTTP + JSON output

```json
{
  "runtime": {
    "host": "127.0.0.1",
    "port": 7777,
    "protocol": "http",
    "output_format": "json",
    "query": null
  },
  "ingestion": {
    "mode": "localdir",
    "corpus_dir": "/home/knny/itcannotalwaysbenight/MATERIALS",
    "s3_bucket": null,
    "s3_prefix": null
  },
  "storage": {
    "db_path": "/home/knny/itcannotalwaysbenight/MATERIALS/zynsearch-index.bin"
  },
  "distribution": {
    "channels": ["http"]
  },
  "cleanup": {
    "enable_periodic_cleanup": true,
    "period_seconds": 60
  },
  "manifest": {
    "name": "zynsearch",
    "version": "1.0.0",
    "description": "Plug-and-play search engine"
  },
  "env_path": null,
  "config_path": "zynsearch.config.json"
}
```

Run:

```bash
cargo build --release -p zynsearch-server
./target/release/zynsearch-server --config ./zynsearch.config.json
```

Search with:

```bash
curl -H "Accept: application/json" "http://127.0.0.1:7777/search?q=machine&limit=5&explain=true"
```

#### Recipe B: One-shot CLI query

```bash
./target/release/zynsearch-cli --config ./zynsearch.config.json --query "engineering"
```

You can also place the query in JSON:

```json
"runtime": {
  "query": "engineering"
}
```

and then run the binary without `--query`.

#### Recipe C: Override everything from the shell

```bash
ZYN_CONFIG_PATH=./zynsearch.config.json \
ZYN_CORPUS_DIR=/home/knny/itcannotalwaysbenight/MATERIALS \
ZYN_FORMAT=json \
./target/release/zynsearch-cli --query "machine"
```

This is useful when you want a reusable config file but different test runs.

<br>

### 7 · Common Mistakes

- Pointing `corpus_dir` at a folder that does not exist
- Forgetting that `curl` requires `runtime.protocol = "http"`
- Leaving an old `db_path` around and expecting a fresh ingest
- Using `s3` mode without setting `s3_bucket`
- Setting only one of `http_tls_cert_path` / `http_tls_key_path`
- Assuming filenames are indexed instead of document contents

<br>

### 8 · Quick Mental Model

Think of the config like this:

| Question | Setting |
|---|---|
| Where do documents come from? | `ingestion` |
| Where does the index live? | `storage.db_path` |
| How do clients connect? | `runtime.protocol` and `distribution.channels` |
| How do results look? | `runtime.output_format` |
| Does the process exit after one query? | `runtime.query` |
| Is the API protected? | `runtime.http_username` and `runtime.http_password` |
| Is transport encrypted? | `runtime.http_tls_cert_path` and `runtime.http_tls_key_path` |

If you can answer those seven questions, you can configure ZynSearch confidently.

<br>

### 9 · Copy-Paste Starter Configs

These are meant to save time when you want to test a specific path quickly.

#### 9.1 CLI test config

Use this when you want a simple one-shot ingest and query run:

```json
{
  "manifest": {
    "name": "zynsearch",
    "version": "1.0.0",
    "description": "Plug-and-play search engine"
  },
  "runtime": {
    "host": "127.0.0.1",
    "port": 7777,
    "protocol": "tcp",
    "output_format": "text",
    "query": "machine",
    "http_username": null,
    "http_password": null,
    "http_tls_cert_path": null,
    "http_tls_key_path": null
  },
  "ingestion": {
    "mode": "localdir",
    "corpus_dir": "/home/knny/itcannotalwaysbenight/MATERIALS",
    "s3_bucket": null,
    "s3_prefix": null
  },
  "storage": {
    "db_path": "/home/knny/itcannotalwaysbenight/MATERIALS/zynsearch-index.bin"
  },
  "distribution": {
    "channels": ["tcp"]
  },
  "cleanup": {
    "enable_periodic_cleanup": true,
    "period_seconds": 60
  },
  "env_path": null,
  "config_path": "zynsearch.config.json"
}
```

Run:

```bash
cargo build --release -p zynsearch-cli
./target/release/zynsearch-cli --config ./zynsearch.config.json
```

#### 9.2 HTTP + curl test config

Use this when you want the server to stay up and accept REST calls:

```json
{
  "manifest": {
    "name": "zynsearch",
    "version": "1.0.0",
    "description": "Plug-and-play search engine"
  },
  "runtime": {
    "host": "127.0.0.1",
    "port": 7777,
    "protocol": "http",
    "output_format": "json",
    "query": null,
    "http_username": null,
    "http_password": null,
    "http_tls_cert_path": null,
    "http_tls_key_path": null
  },
  "ingestion": {
    "mode": "localdir",
    "corpus_dir": "/home/knny/itcannotalwaysbenight/MATERIALS",
    "s3_bucket": null,
    "s3_prefix": null
  },
  "storage": {
    "db_path": "/home/knny/itcannotalwaysbenight/MATERIALS/zynsearch-index.bin"
  },
  "distribution": {
    "channels": ["http"]
  },
  "cleanup": {
    "enable_periodic_cleanup": true,
    "period_seconds": 60
  },
  "env_path": null,
  "config_path": "zynsearch.config.json"
}
```

Run and query:

```bash
cargo build --release -p zynsearch-server
./target/release/zynsearch-server --config ./zynsearch.config.json
curl -H "Accept: application/json" "http://127.0.0.1:7777/search?q=machine&limit=5&explain=true"
```

#### 9.3 gRPC starter config

Use this when you want the gRPC transport only:

```json
{
  "manifest": {
    "name": "zynsearch",
    "version": "1.0.0",
    "description": "Plug-and-play search engine"
  },
  "runtime": {
    "host": "127.0.0.1",
    "port": 7777,
    "protocol": "grpc",
    "output_format": "json",
    "query": null,
    "http_username": null,
    "http_password": null,
    "http_tls_cert_path": null,
    "http_tls_key_path": null
  },
  "ingestion": {
    "mode": "localdir",
    "corpus_dir": "/home/knny/itcannotalwaysbenight/MATERIALS",
    "s3_bucket": null,
    "s3_prefix": null
  },
  "storage": {
    "db_path": "/home/knny/itcannotalwaysbenight/MATERIALS/zynsearch-index.bin"
  },
  "distribution": {
    "channels": ["grpc"]
  },
  "cleanup": {
    "enable_periodic_cleanup": true,
    "period_seconds": 60
  },
  "env_path": null,
  "config_path": "zynsearch.config.json"
}
```

<br>

<div align="center">

*ZynSearch — core, server, and SDKs, kept honest about how they fit together.*

</div>
