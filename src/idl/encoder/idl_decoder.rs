use std::io::Read;

use flate2::read::ZlibDecoder;
use solana_idl::Idl;

use super::IDL_HEADER_SIZE;
use crate::errors::{ChainparserError, ChainparserResult};

/* Related anchor code:
```ts
// Chop off account discriminator.
let idlAccount = decodeIdlAccount(accountInfo.data.slice(8));
const inflatedIdl = inflate(idlAccount.data);
return JSON.parse(utf8.decode(inflatedIdl));

const IDL_ACCOUNT_LAYOUT: borsh.Layout<IdlProgramAccount> = borsh.struct([
  borsh.publicKey("authority"),
  borsh.vecU8("data"),
]);

export function decodeIdlAccount(data: Buffer): IdlProgramAccount {
  return IDL_ACCOUNT_LAYOUT.decode(data);
}

export function vecU8(property?: string): Layout<Buffer> {
  const length = u32("length");
  const layout: Layout<{ data: Buffer }> = struct([
    length,
    blob(offset(length, -length.span), "data"),
  ]);
  return new WrappedLayout(
    layout,
    ({ data }) => data,
    (data) => ({ data }),
    property
  );
}
```
**/

/// Same as [decode_idl_data] except that it strips the prefix bytes before
/// unzipping the packed JSON.
pub fn decode_idl_account_data(
    account_data: &[u8],
) -> ChainparserResult<(Idl, String)> {
    decode_idl_data(&account_data[IDL_HEADER_SIZE..])
}

/// Unzips account data obtained from chain by first stripping the prefix
/// bytes which aren't the zip data and then unpacking the containted string.
pub fn unzip_idl_account_json(bytes: &[u8]) -> ChainparserResult<String> {
    unzip_bytes(&bytes[IDL_HEADER_SIZE..])
}

/// Decodes IDL data by first unzipping the provided data and then parsing
/// the contained JSON.
fn decode_idl_data(data: &[u8]) -> ChainparserResult<(Idl, String)> {
    let json = unzip_bytes(data)?;
    let idl: Idl = serde_json::from_str(&json)?;
    Ok((idl, json))
}

/// Unzips the provided [bytes] into a string.
fn unzip_bytes(bytes: &[u8]) -> ChainparserResult<String> {
    let mut zlib = ZlibDecoder::new(bytes);
    let mut write = String::new();
    zlib.read_to_string(&mut write).map_err(|err| {
        ChainparserError::IdlContainerShouldContainZlibData(err.to_string())
    })?;
    Ok(write)
}
