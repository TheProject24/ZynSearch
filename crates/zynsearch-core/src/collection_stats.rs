use std::sync::atomic::{AtomicU64, Ordering};

pub struct CollectionStats {
    total_documents: AtomicU64,
    total_tokens: AtomicU64,
}

impl CollectionStats {
    pub fn new() -> Self {
        CollectionStats { 
            total_documents: AtomicU64::new(0), 
            total_tokens: AtomicU64::new(0) 
        }
    }

    pub fn add_document(&self, doc_length: u64) {
        self.total_documents.fetch_add(1, Ordering::Relaxed);
        self.total_tokens.fetch_add(doc_length, Ordering::Relaxed);
    }

    pub fn remove_document(&self, doc_length: u64) {
        self.total_documents.fetch_sub(1, Ordering::Relaxed);
        self.total_tokens.fetch_sub(doc_length, Ordering::Relaxed);
    }

    pub fn get_total_documents(&self) -> u64 {
        self.total_documents.load(Ordering::Relaxed)
    }

    pub fn get_avg_document_length(&self) -> f64 {
        let docs = self.get_total_documents();
        if docs == 0 {
            return 0.0;
        }

        let words = self.total_tokens.load(Ordering::Relaxed);

        (words as f64) / (docs as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_statistics_tracking() {
        let stats = Arc::new(CollectionStats::new());
        let mut worker_threads = Vec::new();

        for _ in 0..10 {
            let stats_clone = stats.clone();
            let handle = thread::spawn(move || {
                stats_clone.add_document(100);
            });
            worker_threads.push(handle);
        }

        for handle in worker_threads {
            handle.join().unwrap();
        }

        assert_eq!(stats.get_total_documents(), 10);
        assert_eq!(stats.get_avg_document_length(), 100.0);
        println!("Total DOcs: {}", stats.get_total_documents());
        println!("Average Doc Length: {}", stats.get_avg_document_length());
    }
}