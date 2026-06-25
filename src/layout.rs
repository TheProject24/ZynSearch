fn vbyte_encode_number(mut num: u32) -> Vec<u8> {
    let mut packed_bytes = Vec::new();
    loop {
        let mut piece = (num & 0b0111_1111) as u8;
        num >>= 7;

        if num != 0 {
            piece |= 0b1000_0000;
            packed_bytes.push(piece)
        } else {
            packed_bytes.push(piece);
            break;
        }
    }

    packed_bytes
}

fn vbyte_decode_number(bytes: &[u8], start_index: usize) -> (u32, usize) {
    let mut num: u32 = 0;
    let mut shift_amount = 0;
    let mut bytes_read = 0;

    for i in start_index..bytes.len() {
        let byte = bytes[i];
        bytes_read += 1;

        let data_piece = (byte & 0b111_1111) as u32;
        num |= data_piece << shift_amount;

        if byte < 128 {
            break;
        }

        shift_amount += 7;
    }

    (num, bytes_read)
}

#[derive(Debug, PartialEq)]
pub struct SearchMatch {
    pub doc_delta: u32,
    pub frequency: u32,
}

pub fn pack_train_cars(matches: &[SearchMatch]) -> Vec<u8> {
    let mut full_train = Vec::new();

    for matched_book in matches {
        let mut packed_blue_car = vbyte_encode_number(matched_book.doc_delta);
        let mut packed_red_car = vbyte_encode_number(matched_book.frequency);

        full_train.append(&mut packed_blue_car);
        full_train.append(&mut packed_red_car);
    }

    full_train
}

pub fn unpack_train_cars(train_bytes: &[u8]) -> Vec<SearchMatch> {
    let mut matches = Vec::new();
    let mut current_position = 0;

    while current_position < train_bytes.len() {
        let (doc_delta, bytes_used_for_blue) = vbyte_decode_number(train_bytes, current_position);
        current_position += bytes_used_for_blue;

        let (frequency, bytes_used_for_red) = vbyte_decode_number(train_bytes, current_position);
        current_position += bytes_used_for_red;

        matches.push(SearchMatch {
            doc_delta,
            frequency,
        });
    }

    matches
}