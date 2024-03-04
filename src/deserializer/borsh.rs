use borsh::BorshDeserialize;
use solana_idl::IdlType;
use solana_sdk::pubkey::Pubkey;

use super::{
    floats::{deserialize_f32, deserialize_f64},
    ChainparserDeserialize,
};
use crate::errors::{ChainparserError, ChainparserResult as Result};

#[derive(Clone, Copy)]
pub struct BorshDeserializer;

impl ChainparserDeserialize for BorshDeserializer {
    fn u8(&self, buf: &mut &[u8]) -> Result<u8> {
        u8::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "u8".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn u16(&self, buf: &mut &[u8]) -> Result<u16> {
        u16::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "u16".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn u32(&self, buf: &mut &[u8]) -> Result<u32> {
        u32::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "u32".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn u64(&self, buf: &mut &[u8]) -> Result<u64> {
        u64::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "u64".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn u128(&self, buf: &mut &[u8]) -> Result<u128> {
        u128::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "u128".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn i8(&self, buf: &mut &[u8]) -> Result<i8> {
        i8::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "i8".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn i16(&self, buf: &mut &[u8]) -> Result<i16> {
        i16::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "i16".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn i32(&self, buf: &mut &[u8]) -> Result<i32> {
        i32::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "i32".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn i64(&self, buf: &mut &[u8]) -> Result<i64> {
        i64::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "i64".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn i128(&self, buf: &mut &[u8]) -> Result<i128> {
        i128::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "i128".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn f32(&self, buf: &mut &[u8]) -> Result<f32> {
        deserialize_f32(buf)
    }

    fn f64(&self, buf: &mut &[u8]) -> Result<f64> {
        deserialize_f64(buf)
    }

    fn bool(&self, buf: &mut &[u8]) -> Result<bool> {
        bool::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "bool".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn string(&self, buf: &mut &[u8]) -> Result<String> {
        String::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "String".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn bytes(&self, buf: &mut &[u8]) -> Result<Vec<u8>> {
        Vec::<u8>::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "bytes".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn pubkey(&self, buf: &mut &[u8]) -> Result<Pubkey> {
        Pubkey::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeTypeError(
                "Pubkey".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }

    fn option(&self, buf: &mut &[u8]) -> Result<bool> {
        self.u8(buf).map(|v| v != 0)
    }

    fn coption(&self, _buf: &mut &[u8], _inner: &IdlType) -> Result<bool> {
        Err(ChainparserError::DeserializerDoesNotSupportType(
            "borsh".to_string(),
            "coption".to_string(),
        ))
    }
}
