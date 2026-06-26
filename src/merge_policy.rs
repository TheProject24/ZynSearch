// merge_policy.rs

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MiniSegment {
    pub segment_id: u64,
    pub dictionary: HashMap<String, Vec<u32>>,
}

pub fn merge_small_posters(
    new_segment_id: u64,
    small_segments: Vec<MiniSegment>,
    deleted_documents: &HashSet<u32>,
) -> MiniSegment {
    let mut merged_dictionary: HashMap<String, Vec<u32>> = HashMap::new();

    for poster in small_segments {
        for (word, doc_list) in poster.dictionary {
            let giant_poster_list = merged_dictionary
                .entry(word)
                .or_insert_with(Vec::new);
            
            for doc_id in doc_list {
                if deleted_documents.contains(&doc_id) {
                    continue;
                }

                giant_poster_list.push(doc_id);
            }
        }
    }

    for list in merged_dictionary.values_mut() {
        list.sort_unstable();
        list.dedup();
    }

    MiniSegment { segment_id: new_segment_id, dictionary: merged_dictionary, }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_janitor_merging_and_trash() {
        let mut dict1 = HashMap::new();
        dict1.insert("cat".to_string(), vec![1, 2]);
        let poster1 = MiniSegment{ segment_id: 1, dictionary: dict1 };

        let mut dict2 = HashMap::new();
        dict2.insert("cat".to_string(), vec![3, 4]);
        let poster2 = MiniSegment { segment_id: 2, dictionary: dict2 };

        let pile_of_posters = vec![poster1,poster2];

        let mut trash_can = HashSet::new();
        trash_can.insert(2);

        let giant_poster = merge_small_posters(99, pile_of_posters, &trash_can);

        let cat_list = giant_poster.dictionary.get("cat").unwrap();

        assert_eq!(cat_list, &vec![1,3,4]);

        println!("The Janitpr successfullt merged the files and threw away the trash!");
    }
}