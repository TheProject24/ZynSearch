use std::fs::File;
use std::io::ErrorKind::InvalidData;
use std::io::{Read, Write, BufWriter, BufReader, Result};
use crate::index::{InvertedIndex, Posting};

pub struct StorageManager;

impl StorageManager {
    pub fn serialize(index: &InvertedIndex, file_path: &str) -> Result<()>{
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(b"AURASEARCH")?;

        let doc_count = index.document_registry.len() as u64;
        writer.write_all(&doc_count.to_le_bytes())?;

        for (&doc_id, path) in &index.document_registry {
            writer.write_all(&(doc_id as u64).to_le_bytes())?;
            let path_bytes = path.as_bytes();
            writer.write_all(&(path_bytes.len() as u64).to_le_bytes())?;
            writer.write_all(path_bytes)?;
        }

        let term_count = index.index.len() as u64;
        writer.write_all(&term_count.to_le_bytes())?;

        for (term, posting_list) in &index.index {
            let term_bytes = term.as_bytes();
            writer.write_all(&(term_bytes.len() as u64).to_le_bytes())?;
            writer.write_all(term_bytes)?;

            let posting_count = posting_list.len() as u64;
            writer.write_all(&posting_count.to_le_bytes())?;

            for posting in posting_list {
                writer.write_all(&(posting.document_id as u64).to_le_bytes())?;
                writer.write_all(&posting.frequency.to_le_bytes())?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    pub fn deserialize(file_path: &str) -> Result<InvertedIndex> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        let mut header = [0u8; 10];
        reader.read_exact(&mut header)?;
        if &header != b"AURASEARCH" {
            return Err(std::io::Error::new(
                InvalidData,
                "Invalid AuraSearch database file signature"
            ));
        }

        let mut inverted_index = InvertedIndex::new();

        let mut buf_u64 = [0u8; 8];
        reader.read_exact(&mut buf_u64)?;
        let doc_count = u64::from_le_bytes(buf_u64);

        for _ in 0..doc_count {
            reader.read_exact(&mut buf_u64)?;
            let doc_id = u64::from_le_bytes(buf_u64) as usize;

            reader.read_exact(&mut buf_u64)?;
            let path_len = u64::from_le_bytes(buf_u64) as usize;

            let mut path_bytes = vec![0u8; path_len];
            reader.read_exact(&mut path_bytes)?;
            let path = String::from_utf8(path_bytes)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Malformed UTF-8 string path"))?;

            inverted_index.document_registry.insert(doc_id, path);
        }

        reader.read_exact(&mut buf_u64)?;
        let term_count = u64::from_le_bytes(buf_u64);

        for _ in 0..term_count {
            reader.read_exact(&mut buf_u64)?;
            let term_len = u64::from_le_bytes(buf_u64) as usize;

            let mut term_bytes = vec![0u8; term_len];
            reader.read_exact(&mut term_bytes)?;
            let term = String::from_utf8(term_bytes)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Malformed UTF-8 term key"))?;

            reader.read_exact(&mut buf_u64)?;
            let posting_count = u64::from_le_bytes(buf_u64) as usize;

            let mut posting_list = Vec::with_capacity(posting_count as usize);
            let mut buf_u32 = [0u8; 4];

            for _ in 0..posting_count {
                reader.read_exact(&mut buf_u32)?;
                let doc_id = u64::from_le_bytes(buf_u64) as usize;
                
                reader.read_exact(&mut buf_u32)?;
                let term_frequency = u32::from_le_bytes(buf_u32) as usize;

                posting_list.push(Posting {
                    document_id: doc_id,
                    frequency: term_frequency,
                });
            }

            inverted_index.index.insert(term, posting_list);
        }

        Ok(inverted_index)

    }
}

pub struct ZeroCopyReader <'a> {
    data: &'a [u8],
}

impl<'a> ZeroCopyReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ZeroCopyReader { data }
    }

    pub fn lookup_term_postings(&self, target_term: &str) -> Option<Vec<usize>> {
        let mut cursor = 0;

        if self.data.len() < 10 || &self.data[0..10] != b"AURASEARCH" {
            return None;
        }
        cursor += 10;

        let doc_count = u64::from_le_bytes(self.data[cursor..cursor+8].try_into().ok()?)?;
        cursor += 8;

        for _ in 0..doc_count {
            cursor += 8;
            let path_len = u64::from_le_bytes(self.data[cursor..cursor+8].try_into().ok()?)?; as usize;
            cursor += 8;
            cursor += path_len;
        }

        let term_count = u64::from_le_bytes(self.data[cursor..cursor+8].try_into().ok()?)?;
        cursor += 8;

        for _ in 0..term_count {
            let term_len = u64::from_le_bytes(self.data[cursor..cursor+8].try_into().ok()?)? as usize;
            cursor += 8;

            let current_term_bytes = &self.data[cursor..cursor+term_len]];
            cursor += 8;

            if current_term_bytes == target_term.as_bytes() {
                let mut matched_doc_ids = Vec::with_capacity(posting_count);
                for _ in 0..posting_count {
                    let doc_id = u64::from_le_bytes(self.data[cursor..cursor+8].try_into().ok()?)? as usize;
                    cursor += 8;
                    cursor += 4;

                    matched_doc_ids.push(doc_id);
                }
                return Some(matched_doc_ids);
            } else {
                cursor += posting_count * 12;
            }
        }
        None
    }

}