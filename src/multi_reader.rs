use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MiniSegment {
    pub segment_id: u64,
    pub dictionary: HashMap<String, Vec<u32>>,
}

pub struct MultiSegmentReader {
    pub active_segments: Vec<MiniSegment>,
}

impl MultiSegmentReader {
    pub fn new(segments: Vec<MiniSegment>) -> Self {
        MultiSegmentReader { active_segments: segments }
    }

    pub fn parallel_search(&self, search_word: &str) -> Vec<u32> {
        let target_word = search_word.to_lowercase();

        let final_combined_results: Vec<u32> = self.active_segments
            .into_par_iter()
            .filter_map(|segment| {
                println!("Thread {:?} is checking segment #{} . . .", std::thread::current().id(), segment.segment_id);
                segment.dictionary.get(&target_word).cloned()
            })
            .flatten()
            .collect();

        final_combined_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_ninja_search() {
        let mut seg1_dict = HashMap::new();
        seg1_dict.insert("apple".to_string(), vec![5]);
        let segment_a = MiniSegment { segment_id: 1, dictionary: seg1_dict };

        let seg2_dict = HashMap::new();
        let segment_b = MiniSegment { segment_id: 2, dictionary: seg2_dict };

        let mut seg3_dict = HashMap::new();
        seg3_dict.insert("apple".to_string(), vec![99]);
        let segment_c = MiniSegment { segment_id: 3, dictionary: seg3_dict };

        let manager = MultiSegmentReader::new(vec![segment_a, segment_b, segment_c]);

        let results = manager.parallel_search("apple");

        assert!(results.contains(&5));
        assert!(results.contains(&99));
        assert_eq!(results.len(), 2);


        println!("Final Merged Search Results: {:?}", results);
    }
}