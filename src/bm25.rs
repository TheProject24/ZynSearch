pub struct Bm25Scorer {
    pub k1: f32,
    pub b: f32,
}

impl Bm25Scorer {
    pub fn new() -> Self{
        Bm25Scorer { k1: 1.2, b: 0.75 }
    }

    pub fn calculate_idf(&self, total_documents: u64, doc_with_word: u64) -> f32 {
        let n = total_documents as f32;
        let n_q = doc_with_word as f32;

        let idf_math = (n - n_q + 0.5) / (n_q + 0.5);
        let idf = (1.0 + idf_math).ln();

        if idf < 0.0 {
            0.0
        } else {
            idf   
        }
    }

    pub fn score(
        &self,
        idf: f32,
        term_frequency: u32,
        doc_length: u32,
        average_doc_length: f32,
    ) -> f32 {
        let tf = term_frequency as f32;
        let dl = doc_length as f32;

        let length_ratio = dl / average_doc_length;
        let denom = tf + self.k1 * (1.0 - self.b + self.b * length_ratio);
        let numer = tf * (self.k1 + 1.0);

        let tf_weight = numer/denom;

        idf * tf_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_scoring_logic() {
        let judge = Bm25Scorer::new();
        let total_docs_in_library = 100_000;
        let avg_length = 500.0;
        let rare_idf = judge.calculate_idf(total_docs_in_library, 10);
        let common_idf = judge.calculate_idf(total_docs_in_library, 90_000);

        assert!(rare_idf > common_idf);

        let score_a = judge.score(rare_idf, 3, 100, avg_length);
        let score_b = judge.score(rare_idf, 3, 2000, avg_length);

        assert!(score_a > score_b);

        println!("Tiny Document Score {}", score_a);
        println!("Massive Document Score {}", score_b);
    }
}