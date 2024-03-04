pub mod borsh;
mod floats;
pub mod spl;

use solana_idl::{Idl, IdlType};
use solana_sdk::pubkey::Pubkey;

use crate::errors::ChainparserError;
pub use crate::errors::ChainparserResult as Result;
pub trait ChainparserDeserialize: Clone {
    fn u8(&self, buf: &mut &[u8]) -> Result<u8>;
    fn u16(&self, buf: &mut &[u8]) -> Result<u16>;
    fn u32(&self, buf: &mut &[u8]) -> Result<u32>;
    fn u64(&self, buf: &mut &[u8]) -> Result<u64>;
    fn u128(&self, buf: &mut &[u8]) -> Result<u128>;

    fn i8(&self, buf: &mut &[u8]) -> Result<i8>;
    fn i16(&self, buf: &mut &[u8]) -> Result<i16>;
    fn i32(&self, buf: &mut &[u8]) -> Result<i32>;
    fn i64(&self, buf: &mut &[u8]) -> Result<i64>;
    fn i128(&self, buf: &mut &[u8]) -> Result<i128>;

    fn f32(&self, buf: &mut &[u8]) -> Result<f32>;
    fn f64(&self, buf: &mut &[u8]) -> Result<f64>;

    fn bool(&self, buf: &mut &[u8]) -> Result<bool>;
    fn string(&self, buf: &mut &[u8]) -> Result<String>;

    fn bytes(&self, buf: &mut &[u8]) -> Result<Vec<u8>>;
    fn pubkey(&self, buf: &mut &[u8]) -> Result<Pubkey>;

    fn option(&self, buf: &mut &[u8]) -> Result<bool>;
    fn coption(&self, buf: &mut &[u8], inner: &IdlType) -> Result<bool>;
}

pub enum DeserializeProvider {
    Borsh(borsh::BorshDeserializer),
    Spl(spl::SplDeserializer),
}

impl TryFrom<Option<&str>> for DeserializeProvider {
    type Error = ChainparserError;

    fn try_from(label: Option<&str>) -> std::result::Result<Self, Self::Error> {
        let label = label.unwrap_or("borsh");
        match label {
            "borsh" => Ok(Self::Borsh(borsh::BorshDeserializer)),
            "spl" => Ok(Self::Spl(spl::SplDeserializer::new())),
            _ => Err(ChainparserError::UnsupportedDeserializer(
                label.to_string(),
            )),
        }
    }
}

impl TryFrom<&Idl> for DeserializeProvider {
    type Error = ChainparserError;

    fn try_from(idl: &Idl) -> std::result::Result<Self, Self::Error> {
        let label = idl.metadata.as_ref().and_then(|m| m.serializer.as_deref());
        label.try_into()
    }
}

impl DeserializeProvider {
    pub fn borsh() -> Self {
        Self::Borsh(borsh::BorshDeserializer)
    }

    pub fn is_spl(&self) -> bool {
        matches!(self, DeserializeProvider::Spl(_))
    }

    pub fn is_borsh(&self) -> bool {
        matches!(self, DeserializeProvider::Borsh(_))
    }
}
