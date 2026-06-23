use std::collections::HashSet;
use crate::index::InvertedIndex;
use crate::analyzer::{TextAnalyzer};

pub struct SearchEngine<'a> {
    index: &'a InvertedIndex,
    analyzer: &'a TextAnalyzer,
}

impl <'a> SearchEngine<'a> {
    pub fn new(index: &'a InvertedIndex, analyzer: &'a TextAnalyzer) -> Self {
        SearchEngine { index, analyzer }
    }

    pub fn search(&self, raw_query: &str) -> Vec<String> {
        let query_tokens = self.analyzer.analyze(raw_query);

        if query_tokens.is_empty() {
            return Vec::new();
        }

        let mut matching_doc_ids: HashSet<usize> = Vec::new().into_iter().collect();
        let mut is_first_token = true;

        for token in query_tokens {
            if let Some(posting_list) = self.index.index.get(&token) {
                let current_token_docs: HashSet<usize> = posting_list
                    .iter()
                    .map(|p| p.document_id)
                    .collect();

                if is_first_token {
                    matching_doc_ids = current_token_docs;
                    is_first_token = false;
                } else {
                    matching_doc_ids.retain(|id| current_token_docs.contains(id));
                }
                if matching_doc_ids.is_empty() {
                    return Vec::new();
                }
            } else {
                return Vec::new();
            }
        }

        let mut final_results = Vec::new();
        for doc_id in matching_doc_ids {
            if let Some(file_path) = self.index.document_registry.get(&doc_id) {
                final_results.push(file_path.clone());
            }
        }
        final_results
    }
}