// engine.rs

use std::sync::{Arc, RwLock};
use crate::index::InvertedIndex;
use crate::analyzer::TextAnalyzer;
use crate::searcher::SearchEngine;
use crate::top_k::SearchResult;
use crate::document_ingest;
use crate::crawler::DirectoryCrawler;
use std::path::Path;

pub struct SearchEngineCore {
    pub index: Arc<RwLock<InvertedIndex>>,
    pub analyzer: Arc<TextAnalyzer>,
}

impl SearchEngineCore {
    pub fn new() -> Self {
        SearchEngineCore { 
            index: Arc::new(RwLock::new(InvertedIndex::new())), 
            analyzer: Arc::new(TextAnalyzer::new()) 
        }
    }

    pub fn ingest_document(&self, path: &str, tokens: Vec<String>) {
        let mut write_guard = self.index.write().unwrap();

        let doc_id = write_guard.register_document(path);
        write_guard.add_document(doc_id, tokens);
    }

    pub fn execute_search(&self, raw_query: &str) -> Vec<String> {
        let read_guard = self.index.read().unwrap();
        let searcher = SearchEngine::new(&read_guard, &*self.analyzer);

        searcher.search(raw_query)
    }

    pub fn execute_search_for_shard(
        &self,
        raw_query: &str,
        shard_id: usize,
        shard_count: usize,
        limit: usize,
    ) -> Vec<SearchResult> {
        let read_guard = self.index.read().unwrap();
        let searcher = SearchEngine::new(&read_guard, &*self.analyzer);

        searcher.search_scored(raw_query, shard_id, shard_count, limit)
    }

    pub fn ingest_document_text(&self, path: &str, raw_text: &str) {
        let tokens = self.analyzer.analyze(raw_text);
        self.ingest_document(path, tokens);
    }

    pub fn ingest_corpus_dir(&self, corpus_dir: &Path) -> Result<Vec<String>, String> {
        let crawler = DirectoryCrawler::new(corpus_dir, document_ingest::allowed_extensions());
        let mut indexed = Vec::new();

        for path_buf in crawler.run() {
            let normalized = document_ingest::normalize_for_indexing(&path_buf)?;
            let path_str = path_buf.to_string_lossy().into_owned();
            self.ingest_document_text(&path_str, &normalized);
            indexed.push(path_str);
        }

        Ok(indexed)
    }
}
