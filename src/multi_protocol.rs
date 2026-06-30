use serde::Deserialize;
use std::str;

#[derive(Debug, PartialEq, Deserialize)]
pub struct ZynQuery {
    pub query_string: String,
    pub limit: u32,
}

const MAGIC_BYTE_BINARY: u8 = 0x0B;

pub struct ProtocolParser;

impl ProtocolParser {
    pub fn parse_incoming_payload(payload: &[u8]) -> Result<ZynQuery, String> {
        if payload.is_empty() {
            return Err("Payload is empty! Nothing to search".to_string());
        }

        match payload[0] {
            b'{' => {
                println!("Sniffed a '{{'. Routing to JSON parser . . .");
                Self::parse_json(payload)
            }

            MAGIC_BYTE_BINARY => {
                println!("Sniffed the Magic Byte! Routing to Binary parser . . . ");
                Self::parse_binary(payload)
            }

            _ => {
                println!("No special symbols detected. ROuting to Raw Text parser . . .");
                Self::parse_raw_text(payload)
            }
        }
    }

    fn parse_json(payload: &[u8]) -> Result<ZynQuery, String> {
        serde_json::from_slice::<ZynQuery>(payload).map_err(|e| format!("Invalid JSON format: {}", e))
    }

    fn parse_binary(payload: &[u8]) -> Result<ZynQuery, String> {
        if payload.len() < 5 {
            return Err("Binary payload too short".to_string());
        }

        let limit_bytes: [u8; 4] = payload[1..5].try_into().unwrap();
        let limit = u32::from_be_bytes(limit_bytes);

        let query_bytes = &payload[5..];
        let query_string = str::from_utf8(query_bytes)
            .map_err(|_| "invalid UTF-8 in binary string".to_string())?
            .to_string();

        Ok(ZynQuery { query_string, limit })
    }

    fn parse_raw_text(payload: &[u8]) -> Result<ZynQuery, String> {
        let raw_string = str::from_utf8(payload)
            .map_err(|_| "Text was not valid UTF-8 format".to_string())?;

        Ok(ZynQuery { query_string: raw_string.trim().to_string(), limit: 10 })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplexer_routing() {
        let json_payload = b"{\"query_string\": \"fast car\", \"limit\": 5}";
        let parsed_json = ProtocolParser::parse_incoming_payload(json_payload).unwrap();

        assert_eq!(parsed_json.query_string, "fast car");
        assert_eq!(parsed_json.limit, 5);

        let text_payload = b"fast car";
        let parsed_text = ProtocolParser::parse_incoming_payload(text_payload).unwrap();

        assert_eq!(parsed_text.query_string, "fast car");
        assert_eq!(parsed_text.limit, 10);

        let mut binary_payload = Vec::new();
        binary_payload.push(MAGIC_BYTE_BINARY);
        binary_payload.extend_from_slice(&20u32.to_be_bytes());
        binary_payload.extend_from_slice(b"fast car");

        let parsed_binary = ProtocolParser::parse_incoming_payload(&binary_payload).unwrap();

        assert_eq!(parsed_binary.query_string, "fast car");
        assert_eq!(parsed_binary.limit, 20);

        println!("Mailroom successfully routed all three formats into identical ZynQueries!");
    }
}