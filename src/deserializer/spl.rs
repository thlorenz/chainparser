use solana_idl::IdlType;
use solana_sdk::pubkey::Pubkey;
use TryFrom;

use super::{borsh::BorshDeserializer, ChainparserDeserialize};
use crate::{
    errors::{ChainparserError, ChainparserResult as Result},
    idl,
};

#[derive(Clone, Copy)]
pub struct SplDeserializer {
    borsh: BorshDeserializer,
}

impl SplDeserializer {
    pub(crate) fn new() -> Self {
        Self {
            borsh: BorshDeserializer,
        }
    }
}

impl ChainparserDeserialize for SplDeserializer {
    fn u8(&self, buf: &mut &[u8]) -> Result<u8> {
        self.borsh.u8(buf)
    }

    fn u16(&self, buf: &mut &[u8]) -> Result<u16> {
        self.borsh.u16(buf)
    }

    fn u32(&self, buf: &mut &[u8]) -> Result<u32> {
        self.borsh.u32(buf)
    }

    fn u64(&self, buf: &mut &[u8]) -> Result<u64> {
        self.borsh.u64(buf)
    }

    fn u128(&self, buf: &mut &[u8]) -> Result<u128> {
        self.borsh.u128(buf)
    }

    fn i8(&self, buf: &mut &[u8]) -> Result<i8> {
        self.borsh.i8(buf)
    }

    fn i16(&self, buf: &mut &[u8]) -> Result<i16> {
        self.borsh.i16(buf)
    }

    fn i32(&self, buf: &mut &[u8]) -> Result<i32> {
        self.borsh.i32(buf)
    }

    fn i64(&self, buf: &mut &[u8]) -> Result<i64> {
        self.borsh.i64(buf)
    }

    fn i128(&self, buf: &mut &[u8]) -> Result<i128> {
        self.borsh.i128(buf)
    }

    fn f32(&self, buf: &mut &[u8]) -> Result<f32> {
        self.borsh.f32(buf)
    }

    fn f64(&self, buf: &mut &[u8]) -> Result<f64> {
        self.borsh.f64(buf)
    }

    fn bool(&self, buf: &mut &[u8]) -> Result<bool> {
        self.borsh.bool(buf)
    }

    fn string(&self, buf: &mut &[u8]) -> Result<String> {
        self.borsh.string(buf)
    }

    fn bytes(&self, buf: &mut &[u8]) -> Result<Vec<u8>> {
        self.borsh.bytes(buf)
    }

    fn pubkey(&self, buf: &mut &[u8]) -> Result<Pubkey> {
        let key = &buf[0..32];
        let res = Pubkey::try_from(key).map_err(|e| {
            ChainparserError::TryFromSliceError(
                "pubkey".to_string(),
                e,
                buf.to_vec(),
            )
        })?;
        *buf = &buf[32..];
        Ok(res)
    }

    fn option(&self, _buf: &mut &[u8]) -> Result<bool> {
        Err(ChainparserError::DeserializerDoesNotSupportType(
            "spl".to_string(),
            "option".to_string(),
        ))
    }

    fn coption(&self, buf: &mut &[u8], inner: &IdlType) -> Result<bool> {
        if buf.len() < 4 {
            return Err(ChainparserError::InvalidDataToDeserialize(
                "coption".to_string(),
                "buf too short".to_string(),
                buf.to_vec(),
            ));
        }

        let tag = &buf[0..4];
        *buf = &buf[4..];
        match *tag {
            [0, 0, 0, 0] => {
                // COption is constant size, meaning None and Some take the same space.
                // In case of None it is filled with `0`s. Therefore in order to know
                // how far to consume the buffer we need to know the size of the inner
                // type without deserializing its data.

                // TODO(thlorenz): need the type_map here in order to pass it to idl_type_bytes to
                // resolve defined types, otherwise we can't deserialize COption with defined types
                // as inner
                if let Some(byte_len) = idl::idl_type_bytes(inner, None) {
                    *buf = &buf[byte_len..];
                    Ok(false)
                } else {
                    Err(ChainparserError::InvalidDataToDeserialize(
                        "coption".to_string(),
                        "byte size of inner type needs to be known when it is None"
                            .to_string(),
                        buf.to_vec(),
                    ))
                }
            }
            [1, 0, 0, 0] => Ok(true),
            _ => Err(ChainparserError::InvalidDataToDeserialize(
                "coption".to_string(),
                "invalid tag".to_string(),
                tag.to_vec(),
            )),
        }
    }
}
