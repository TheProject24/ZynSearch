// src/bm_wand.rs

use std::usize;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockMaxSignpost {
    pub max_doc_id: u32,
    pub max_score: f32,
    pub byte_offset: usize,
}

pub struct TopResultsLeaderBoard {
    scores: Vec<f32>,
    max_winners: usize,
}

impl TopResultsLeaderBoard {
    pub fn new(max_winners: usize) -> Self {
        TopResultsLeaderBoard { scores: Vec::new(), max_winners }
    }

    pub fn get_threshold_to_beat(&self) -> f32 {
        if self.scores.len() < self.max_winners {
            return 0.0;
        }

        self.scores[0]
    }

    pub fn try_add_score(&mut self, new_score: f32) {
        let threshold = self.get_threshold_to_beat();

        if self.scores.len() == self.max_winners && new_score <= threshold {
            return;
        }

        self.scores.push(new_score);
        self.scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

        if self.scores.len() > self.max_winners {
            self.scores.remove(0);
        }
    }
}

pub fn block_max_wand_search(
    signposts: &[BlockMaxSignpost],
    leaderboard: &mut TopResultsLeaderBoard,
) {
    for (block_number, signpost) in signposts.iter().enumerate() {
        let threshold = leaderboard.get_threshold_to_beat();

        if signpost.max_score <= threshold {
            println!("Block {} Skipped! Max Possible Score ({}) cannot beat threshold ({}).",
                        block_number, signpost.max_score, threshold
                    );

                    continue;
        }

        println!("Block {} accepted! Diving into the binary bytes to calculate exact scores . . .", block_number);

        let mock_calculated_score = signpost.max_score - 1.0;
        leaderboard.try_add_score(mock_calculated_score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_max_wand() {
        let mut leaderboard - TopResultsLeaderBoard::new(2);

        let piles = vec![
            BlockMaxSignpost { max_doc_id: 128, max_score: 5.0, byte_offset: 0 },
            BlockMaxSignpost { max_doc_id: 256, max_score: 20.0, byte_offset: 100 },
            BlockMaxSignpost { max_doc_id: 384, max_score: 3.0, byte_offset: 200 },
            BlockMaxSignpost { max_doc_id: 512, max_score: 15.0, byte_offset: 300 },
        ];

        block_max_wand_search(&piles, &mut leaderboard);

        let final_threshold = leaderboard.get_threshold_to_beat();

        assert_eq!(final_threshold, 14.0);
        println!("Fianl score to beat to get on the board: {}", final_threshold);
    }
}