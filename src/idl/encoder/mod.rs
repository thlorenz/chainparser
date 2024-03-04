mod idl_decoder;
mod idl_encoder;

pub use idl_decoder::*;
pub use idl_encoder::*;

// anchor:cli/src/lib.rs
pub const IDL_HEADER_SIZE: usize = 44;

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose, Engine as _};
    use solana_idl::Idl;
    use solana_sdk::pubkey::Pubkey;

    use super::*;

    pub fn base64_decode(data: &str) -> Vec<u8> {
        general_purpose::STANDARD.decode(data).unwrap()
    }

    // "{\"version\":\"4.15.7\",\"name\":\"minimal\",\"instructions\":[]}";
    const BASE64_ENCODED_ZIP_OF_IDL_JSON: &str =
    "GEZivzqQe57cMppXObv8MtuJT6/acEkBfTCpZ95AxHl8kvhuCiZU4j0AAAB4nAEyAM3/eyJ2ZXJzaW9uIjoiMC4xLjAiLCJuYW1lIjoiZm9vIiwiaW5zdHJ1Y3Rpb25zIjpbXX2IfRAYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

    fn get_basic_idl() -> Vec<u8> {
        base64_decode(BASE64_ENCODED_ZIP_OF_IDL_JSON.trim_end())
    }

    #[test]
    fn unzip_json_from_basic_idl_account_and_reencode() {
        let idl_account_data = get_basic_idl();
        let json = unzip_idl_account_json(&idl_account_data)
            .expect("should unzip to JSON");
        assert_eq!(
            json,
            "{\"version\":\"0.1.0\",\"name\":\"foo\",\"instructions\":[]}"
        );
        let some_pubkey = Pubkey::new_unique();
        let encoded = encode_idl_account_json(&some_pubkey, &json).unwrap();
        assert_eq!(
            encoded[IDL_HEADER_SIZE..],
            idl_account_data[IDL_HEADER_SIZE..encoded.len()]
        );
    }

    #[test]
    fn roundtrip_minimal_idl() {
        const BASIC_IDL_JSON: &str =
            "{\"version\":\"0.1.0\",\"name\":\"foo\",\"instructions\":[]}";

        let some_pubkey = Pubkey::new_unique();
        let idl: Idl = serde_json::from_str(BASIC_IDL_JSON).unwrap();
        let encoded = encode_idl_account(&some_pubkey, &idl).unwrap();
        let (decoded_idl, decoded_json) =
            decode_idl_account_data(&encoded).unwrap();

        assert_eq!(decoded_idl, idl);
        assert_eq!(decoded_json, BASIC_IDL_JSON);
    }
}
