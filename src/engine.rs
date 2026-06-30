// engine.rs

use std::sync::{Arc, RwLock};
use crate::index::InvertedIndex;
use crate::analyzer::TextAnalyzer;
use crate::searcher::SearchEngine;
use crate::top_k::SearchResult;

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
}
