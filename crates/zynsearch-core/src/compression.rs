// use std::{io::Bytes, vec};

pub fn encode_deltas(ids: &[u32]) -> Vec<u32> {
    if ids.is_empty() {
        return Vec::new();
    }

    let mut deltas = Vec::with_capacity(ids.len());
    deltas.push(ids[0]);

    for i in 1..ids.len() {
        let jump = ids[i] - ids[i - 1];

        deltas.push(jump);
    }

    deltas
}

pub fn decode_deltas(deltas: &[u32]) -> Vec<u32> {
    if deltas.is_empty() {
        return Vec::new();
    }

    let mut ids = Vec::with_capacity(deltas.len());

    ids.push(deltas[0]);

    for i in 1..deltas.len() {
        let true_id = ids[i - 1] + deltas[i];
        ids.push(true_id);
    }

    ids
}

pub fn vbyte_encode(mut num: u32) -> Vec<u8> {
    let mut packed_bytes = Vec::new();

    loop {
        let mut piece = (num & 0b0111_1111) as u8;

        num >>= 7;

        if num != 0 {
            piece |= 0b1000_0000;
            packed_bytes.push(piece);
        } else {
            packed_bytes.push(piece);
            break;
        }
    }

    packed_bytes
}

pub fn vbyte_decode(bytes: &[u8]) -> (u32, usize) {
    let mut num: u32 = 0;
    let mut shift_amount = 0;
    let mut bytes_read = 0;

    for &byte in bytes {
        bytes_read += 1;

        let data_piece = (byte & 0b0111_1111) as u32;
        num |= data_piece << shift_amount;

        if byte < 128 {
            break;
        }

        shift_amount += 7;
    }

    (num, bytes_read)
}

fn bits_required(num: u32) -> u8 {
    if num == 0 {
        return 0;
    }

    (32 - num.leading_zeros()) as u8
}

pub fn encode_for_block(block: &[u32]) -> Vec<u8> {
    if block.is_empty() {
        return Vec::new();
    }

    let max_val = *block.iter().max().unwrap_or(&0);
    let bit_width = bits_required(max_val);
    let mut packed_data = Vec::new();

    packed_data.push(bit_width);

    if bit_width == 0 {
        return packed_data;
    }

    let mut current_byte = 0u8;
    let mut bits_in_current_byte = 0;

    for &num in block {
        for i in 0..bit_width {
            let bit = (num >> i) & 1;

            current_byte |= (bit as u8) << bits_in_current_byte;
            bits_in_current_byte += 1;

            if bits_in_current_byte == 8 {
                packed_data.push(current_byte);
                current_byte = 0;
                bits_in_current_byte = 0;
            }
        }
    }

    if bits_in_current_byte > 0 {
        packed_data.push(current_byte);
    }

    packed_data

}

pub fn decode_for_block(packed: &[u8], block_size: usize) -> Vec<u32> {
    if packed.is_empty() || block_size == 0 {
        return Vec::new();
    }

    let bit_width = packed[0];

    let mut block = Vec::with_capacity(block_size);

    if bit_width == 0 {
        for _ in 0..block_size{
            block.push(0);
        }

        return block;
    }

    let mut byte_index = 1;
    let mut bits_read_from_byte = 0;

    for _ in 0..block_size {
        let mut num = 0u32;

        for i in 0..bit_width {
            let current_byte = packed[byte_index];

            let bit = (current_byte >> bits_read_from_byte) & 1;

            num |= (bit as u32) << i;
            bits_read_from_byte += 1;

            if bits_read_from_byte == 8 {
                byte_index += 1;
                bits_read_from_byte = 0;
            }
        }

        block.push(num);
    }

    block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pallet_packing() {
        let jumps = vec![4,2,7,1,5];
        let packed = encode_for_block(&jumps);

        assert_eq!(packed.len(), 3);

        let unpacked = decode_for_block(&packed, jumps.len());
        assert_eq!(unpacked, jumps);
    }

    #[test]
    fn test_staircase_jumps() {
        let original_docs = vec![1000, 1004, 1012];

        let compressed = encode_deltas(&original_docs);
        assert_eq!(compressed, vec![1000, 4, 8]);

        let decompressed = decode_deltas(&compressed);
        assert_eq!(decompressed, original_docs);
    }

    #[test]
    fn test_moving_box_shrinking() {
        let socks = 8;
        let packed = vbyte_encode(socks);
        assert_eq!(packed.len(), 1);

        let (unpacked, _) = vbyte_decode(&packed);
        assert_eq!(unpacked, socks);

        let big_tv = 300;
        let packed_tv = vbyte_encode(big_tv);
        assert_eq!(packed_tv.len(), 2);

        let (unpacked_tv, _) = vbyte_decode(&packed_tv);
        assert_eq!(unpacked_tv, big_tv);
    }
}