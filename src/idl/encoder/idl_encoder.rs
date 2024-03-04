use std::io::Write;

use flate2::write::ZlibEncoder;
use solana_idl::Idl;
use solana_sdk::pubkey::Pubkey;

use crate::errors::ChainparserResult;

/*
* Structure of an Anchor IDL account:

```rust
#[derive(Debug)]
pub struct IdlAccount {
    // Address that can modify the IDL.
    pub authority: Pubkey,
    // Length of compressed idl bytes.
    pub data_len: u32,
    // Followed by compressed idl bytes.
}
```

   Discriminator 8 bytes (always the same)

/* 0000: */ 0x18, 0x46, 0x62, 0xbf,
/* 0004: */ 0x3a, 0x90, 0x7b, 0x9e,

   Struct start ...
   Pubkey of IDL program (Address that can modify the IDL) (32 bytes), example:

/* 0008: */ 0x06, 0x9b, 0xe9, 0xd3,
/* 0012: */ 0x33, 0x01, 0x1c, 0x15,
/* 0016: */ 0x29, 0x75, 0x23, 0x79,
/* 0020: */ 0xd8, 0xf5, 0x04, 0x6d,
/* 0024: */ 0xcd, 0x15, 0x7d, 0xe3,
/* 0028: */ 0x10, 0xbe, 0x9e, 0x04,
/* 0032: */ 0xb0, 0x31, 0x31, 0xcb,

   Data len of data (before compression, i.e. Vec::len()) (4 bytes), example:

/* 0036: */ 0xea, 0x71, 0xd9, 0x7e,
/* 0040: */ 0x6c, 0x01, 0x00, 0x00,

   ZLib header: always 0x78 0x9c

/* 0044: */ 0x78, 0x9c,
...data
*/

#[rustfmt::skip]
const DISCRIMINATOR: [u8; 8] = [
    0x18, 0x46, 0x62, 0xbf,
    0x3a, 0x90, 0x7b, 0x9e,
];

pub fn encode_idl_account(
    program_id: &Pubkey,
    idl: &Idl,
) -> ChainparserResult<Vec<u8>> {
    let json = serde_json::to_vec(idl)?;

    let pubkey_vec = program_id.to_bytes().to_vec();
    let data_len_bytes = (json.len() as u32).to_le_bytes().to_vec();
    let zipped = zip_bytes(&json)?;

    let full_vec =
        [DISCRIMINATOR.to_vec(), pubkey_vec, data_len_bytes, zipped].concat();
    Ok(full_vec)
}

pub fn encode_idl(idl: &Idl) -> ChainparserResult<Vec<u8>> {
    let json = serde_json::to_vec(idl)?;
    zip_bytes(&json)
}

pub fn encode_idl_account_json(
    program_id: &Pubkey,
    idl_json: &str,
) -> ChainparserResult<Vec<u8>> {
    let json_bytes = idl_json.as_bytes();
    let pubkey_vec = program_id.to_bytes().to_vec();
    let data_len_bytes = (json_bytes.len() as u32).to_le_bytes().to_vec();
    let zipped = zip_bytes(idl_json.as_bytes())?;

    let full_vec =
        [DISCRIMINATOR.to_vec(), pubkey_vec, data_len_bytes, zipped].concat();
    Ok(full_vec)
}

fn zip_bytes(bytes: &[u8]) -> ChainparserResult<Vec<u8>> {
    let mut encoder =
        ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}
