# AuraSearch Roadmap

## Phase 1: Storage Layer & Bit-Packed Compression

Standard 32-bit integer arrays will choke your CPU cache lines. The goal here is to squeeze posting lists down to the bit level.
//TODO!
- [x] Implement delta encoding (d-gaps): store document IDs as incremental differences, such as `[1000, 1004, 1012]` becoming `[1000, 4, 8]`, to shrink integer storage.
//TODO!
- [x] Build a frame-of-reference (FOR) encoder: group document IDs into 128-byte blocks and compress them based on the maximum bits required for the largest value in each block.
//TODO!
- [x] Integrate Roaring Bitmaps: dynamically swap out FOR compression for highly optimized bitwise bitmaps when handling terms with exceptionally dense document matches.
//TODO!
- [x] Design a variable-byte (VByte) layout: use bit shifting to compress integers such as term frequencies and payload lengths so small values use 1 byte instead of 4 or 8.

## Phase 2: Lucene-Style Segment Architecture

To eliminate read locks and maximize throughput, the storage engine should move from a single file to an immutable segment architecture.
//TODO!
- [x] Implement a write-ahead log (WAL): log raw document write operations to disk before executing them in memory to improve persistence guarantees.
//TODO!
- [x] Build an in-memory buffer (MemTable): collect incoming document batches in a lock-free buffer structure before writing them to disk.
//TODO!
- [x] Design the immutable segment writer: flush the memory buffer to disk as an unchangeable, independent mini inverted index once it reaches a size threshold.
//TODO!
- [ ] Develop a multi-segment reader layer: update search execution logic to query all active on-disk segments concurrently using a work-stealing thread pool such as `rayon`.
//TODO!
- [ ] Write a tiered segment merge policy: build a background worker that continuously monitors small segments and merges them into larger consolidated structures while cleaning up deleted document tags.

## Phase 3: Logarithmic Query Traversal Algorithms

Linear text matching scales poorly. The search layer needs structures that can skip straight to the answer.

//TODO!
- [ ] Construct a finite state transducer (FST) dictionary: compress the vocabulary into a deterministically compiled byte array so lookups remain cache-friendly.
//TODO!
- [ ] Embed skip lists in postings blocks: add navigation offsets every 128 document IDs inside the binary schema to speed up set intersection.
//TODO!
- [ ] Implement the Block-Max WAND algorithm: use internal score trackers within postings blocks to skip evaluating document sequences that cannot beat the current top results.
//TODO!
- [ ] Support Boolean query operators: expand the execution core to parse complex query conditions beyond basic sequences, including `MUST`, `SHOULD`, and `MUST_NOT` clauses.

## Phase 4: Relevance Engine & Ranking Mechanics

Speed means little if the first returned result is irrelevant. A statistical ranking layer will make the engine much more useful.

//TODO!
- [ ] Add in-memory collection statistics: maintain real-time tracking metrics such as total document counts and average document length.
//TODO!
- [ ] Build a native BM25 scorer module: calculate document weights dynamically using term frequency and document-length normalization.
//TODO!
- [ ] Implement bounded min-heaps: stream document IDs through a small priority queue to extract only the top `K` results instead of sorting everything.
//TODO!
- [ ] Support positional indexing: store positional byte offsets inside posting payloads to allow exact matching for phrase queries.

## Phase 5: Network Protocols & Cluster Sharding

To handle production traffic at scale, the single node should support network protocols and distribution across multiple machines.

//TODO!
- [ ] Design a length-prefixed binary wire framing: prepend a 4-byte big-endian length header to outgoing socket operations to isolate TCP data packages cleanly.
//TODO!
- [ ] Expose a multi-protocol socket API: let clients toggle execution parameters via raw text strings, JSON requests, or binary payloads.
//TODO!
- [ ] Implement data partitioning (sharding): distribute document indices across independent cluster instances by hashing incoming source ID values.
//TODO!
- [ ] Construct a scatter-gather query coordinator: route incoming queries across multiple search nodes simultaneously and combine the results into a unified sorted response list.

