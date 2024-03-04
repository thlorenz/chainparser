mod encoder;
mod idl_address;
mod idl_provider;
mod idl_retriever;

use std::fmt;

pub use encoder::*;
pub use idl_address::*;
pub use idl_provider::*;
pub use idl_retriever::*;

/// The provider responsible for generating the IDL.
/// Some providers like [Anchor] also prefix the account data in a specific way, i.e. by adding a
/// discriminator
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IdlProvider {
    Anchor,
    Shank,
}

pub const IDL_PROVIDERS: &[IdlProvider; 2] =
    &[IdlProvider::Anchor, IdlProvider::Shank];

impl TryFrom<&str> for IdlProvider {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "anchor" => Ok(Self::Anchor),
            "shank" => Ok(Self::Shank),
            _ => Err(()),
        }
    }
}

impl fmt::Display for IdlProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anchor => write!(f, "anchor"),
            Self::Shank => write!(f, "shank"),
        }
    }
}
