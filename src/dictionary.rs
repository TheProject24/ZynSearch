// src/dictionary.rs

use fst::{Map, MapBuilder};
use std::error::Error;

pub struct FstDictionary {
    compiled_maze: Map<Vec<u8>>,
}

impl FstDictionary {
    pub fn compile_from_sorted_words(mut data: Vec<(String, u64)>) -> Result<Self, Box<dyn Error>> {
        data.sort_by_key(|(word, _)| word.clone());

        let mut maze_bytes = Vec::new();

        let mut builder = MapBuilder::new(&mut maze_bytes)?;

        for (word, posting_list_id) in data {
            builder.insert(&word, posting_list_id)?;
        }

        builder.finish()?;

        let compiled_maze = Map::new(maze_bytes)?;

        Ok(FstDictionary { compiled_maze })
    }

    pub fn lookup_word(&self, word: &str) -> Option<u64> {
        self.compiled_maze.get(word)
    }

    pub fn size_in_bytes(&self) -> usize {
        self.compiled_maze.as_fst().as_bytes().len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_word_maze_lookup() {
        let raw_vocab = vec![
            ("apple".to_string(), 101),
            ("car".to_string(), 102),
            ("cat".to_string(), 103),
            ("cats".to_string(), 104),
        ];

        let dict = FstDictionary::compile_from_sorted_words(raw_vocab).unwrap();

        assert_eq!(dict.lookup_word("cat"), Some(103));
        assert_eq!(dict.lookup_word("cats"), Some(104));
        assert_eq!(dict.lookup_word("wizard"), None);

        println!("Our entire vocab maze takes only {} bytes!", dict.size_in_bytes());
    }
}