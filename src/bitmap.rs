// bitmap.rs

use roaring::RoaringBitmap;

pub struct SmartPostingList {
    bitmap: RoaringBitmap,
}

impl SmartPostingList {
    pub fn new() -> Self {
        SmartPostingList { bitmap: RoaringBitmap::new() }
    }

    pub fn add_document(&mut self, doc_id: u32) {
        self.bitmap.insert(doc_id);
    }

    pub fn add_many_documents(&mut self, doc_ids: &[u32]) {
        self.bitmap.extend(doc_ids.iter().cloned());
    }

    pub fn contains(&self, doc_id: u32) -> bool {
        self.bitmap.contains(doc_id)
    }

    pub fn to_normal_list(&self) -> Vec<u32> {
        self.bitmap.iter().collect()
    }

    pub fn save_to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let _ = self.bitmap.serialize_into(&mut buffer);
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roaring_bitmap_magic() {
        let mut smart_list = SmartPostingList::new();

        smart_list.add_many_documents(&[5, 102, 999]);

        let dense_docs: Vec<u32> = (2000..12000).collect();
        smart_list.add_many_documents(&dense_docs);

        assert!(smart_list.contains(102));
        assert!(smart_list.contains(5000));
        assert!(!smart_list.contains(99999));

        let packed_bytes = smart_list.save_to_bytes();

        println!("10, 000+ numbers packed into just {} bytes!", packed_bytes.len());
    }
}