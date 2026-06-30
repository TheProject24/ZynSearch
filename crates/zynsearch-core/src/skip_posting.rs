// src/skip_posting.rs

#[derive(Debug, Clone, PartialEq)]
pub struct SkipPointer {
    pub max_doc_id: u32,
    pub byte_offset: usize,
}

#[derive(Debug)]
pub struct EncodedPostingWithSkips {
    pub binary_payload: Vec<u8>,
    pub skip_table: Vec<SkipPointer>,
}

pub fn encode_postings_with_skip_pointers(doc_ids: &[u32]) -> EncodedPostingWithSkips {
    let mut binary_payload = Vec::new();
    let mut skip_table = Vec::new();

    let  block_size = 128;

    for chunk in doc_ids.chunks(block_size) {
        let current_block_offset = binary_payload.len();
        let max_id_in_block = *chunk.last().unwrap_or(&0);

        skip_table.push(SkipPointer {
            max_doc_id: max_id_in_block,
            byte_offset: current_block_offset,
        });

        for &doc_id in chunk {
            binary_payload.extend_from_slice(&doc_id.to_le_bytes());
        }
    }

    EncodedPostingWithSkips {
        binary_payload,
        skip_table
    }
}

pub fn fast_skip_search(encoded: &EncodedPostingWithSkips, target_doc_id: u32) -> bool {
    let mut current_skip_index = 0;

    while current_skip_index < encoded.skip_table.len() {
        let signpost = &encoded.skip_table[current_skip_index];

        if signpost.max_doc_id < target_doc_id {
            println!("Skipping 128 documents instantly! Max ID {} is too low", signpost.max_doc_id);
            current_skip_index += 1;
        } else {
            println!("Target might be here! Diving into binary payload at byte position: {}", signpost.byte_offset);
            let start_byte = signpost.byte_offset;

            let end_byte = std::cmp::min(start_byte + (128 * 4), encoded.binary_payload.len());
            let block_bytes = &encoded.binary_payload[start_byte..end_byte];

            for chunk in block_bytes.chunks_exact(4) {
                let doc_id = u32::from_le_bytes(chunk.try_into().unwrap());
                if doc_id == target_doc_id {
                    return true;
                }
            }
            break;
        }   
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_list_traversal() {
        let mut big_doc_list = Vec::new();
        for i in 1..=300 {
            big_doc_list.push(i);
        }

        let encoded_data = encode_postings_with_skip_pointers(&big_doc_list);

        assert_eq!(encoded_data.skip_table.len(), 3);
        let found = fast_skip_search(&encoded_data, 270);

        assert!(found);
    }
}