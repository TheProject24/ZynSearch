// segment.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Segment {
    pub dictionary: HashMap<String, Vec<u32>>,
    pub total_docs: usize,
}

impl Segment {
    pub fn flush_to_disk(
        segment_id: u64,
        memtable_data: &[(u32, String)],
        storage_folder: &PathBuf,
    ) -> std::io::Result<()> {
        let mut new_segment = Segment {
            dictionary: HashMap::new(),
            total_docs: memtable_data.len()
        };

        for (doc_id, content) in memtable_data {
            let words = content.to_lowercase();
            let tokens = words.split_whitespace();

            for token in tokens {
                new_segment
                    .dictionary
                    .entry(token.to_string())
                    .or_insert_with(Vec::new)
                    .push(*doc_id);
            }
        }

        let file_name = format!("segment_{}.bin", segment_id);
        let file_path = storage_folder.join(file_name);

        let mut file = File::create(file_path)?;

        file.sync_all()?;

        println!("Succeddfully laminated Segment #{} to disk!", segment_id);

        Ok(())
    }
}