// analyzer.rs

use std::collections::HashSet;

pub struct TextAnalyzer {
    stop_words: HashSet<String>,
}

impl TextAnalyzer {
    pub fn new() -> Self {
        let default_stopwords = vec![
            "the", "is", "in", "at", "which", "on", "a", "an", 
            "to", "be", "it", "of", "and", "or", "but", "not", 
            "with", "for", "as", "by", "this", "that"
        ];

        let stop_words_set: HashSet<String> = default_stopwords
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        TextAnalyzer { stop_words: stop_words_set }
    }

    pub fn analyze(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();

        let normalized_text = text.to_lowercase();

        let raw_slices = normalized_text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation());

        for token_slice in raw_slices {
            let clean_token = token_slice.trim();

            if clean_token.is_empty() || clean_token.len() < 2 {
                continue;
            }

            if self.stop_words.contains(clean_token) {
                continue;
            }

            let stemmed_token = self.apply_heuristic_stemming(clean_token);

            tokens.push(stemmed_token);
        }
        tokens
    }

    fn is_stop_word(&self, word: &str) -> bool {
        for stopper in &self.stop_words {
            if word == stopper {
                return true;
            }
        }

        false
    }

    fn apply_heuristic_stemming(&self, word: &str) -> String {
        if let Some(root) = word.strip_suffix("ing") {
            if root.len() >= 2 { return root.to_string(); }
        }

        if let Some(root) = word.strip_suffix("ed") {
            if root.len() >= 2 { return root.to_string(); }
        }

        if let Some(root) = word.strip_suffix("ness") {
            if root.len() >= 2 { return root.to_string(); }
        }

        if let Some(root) = word.strip_suffix("s") {
            if !word.ends_with("ss") {
                if let Some(inner_root) = root.strip_suffix("e") {
                    if inner_root.len() >= 2 { return inner_root.to_string(); };
                }
                return root.to_string();
            }
        }

        word.to_string()
    }
}