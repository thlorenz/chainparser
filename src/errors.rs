use thiserror::Error;

pub type ChainparserResult<T> = Result<T, ChainparserError>;

#[derive(Error, Debug)]
pub enum ChainparserError {
    #[error("Format Error")]
    FormatError(#[from] std::fmt::Error),

    #[error("Borsh IO Error")]
    BorshIoError(#[from] borsh::maybestd::io::Error),

    #[error("Deserializer '{0}' is not supported by chainsaw")]
    UnsupportedDeserializer(String),

    #[error("Borsh failed to deserialize type '{1}' ({0}")]
    BorshDeserializeTypeError(String, borsh::maybestd::io::Error, Vec<u8>),

    #[error("Borsh failed to deserialize float '{1}' ({0} {2:?}")]
    BorshDeserializeFloatError(String, borsh::maybestd::io::Error, Vec<u8>),

    #[error("Chainparser failed to deserialize type '{0}' ({1} {2:?}")]
    TryFromSliceError(String, std::array::TryFromSliceError, Vec<u8>),

    #[error("Borsh failed to deserialize type '{0}' ({1})")]
    CompositeDeserializeError(String, Box<ChainparserError>),

    #[error("Borsh failed to deserialize type for field '{0}' ({1})")]
    FieldDeserializeError(String, Box<ChainparserError>),

    #[error("Borsh failed to deserialize type for enum variant '{0}' ({1})")]
    EnumVariantDeserializeError(String, Box<ChainparserError>),

    #[error("Borsh failed to deserialize type for struct '{0}' ({1})")]
    StructDeserializeError(String, Box<ChainparserError>),

    #[error("Borsh failed to deserialize type for enum '{0}' ({1})")]
    EnumDeserializeError(String, Box<ChainparserError>),

    #[error("The '{0}' deserializer does not support type '{1}'")]
    DeserializerDoesNotSupportType(String, String),

    #[error(
        "Encountered '{1}' when trying to deserizalize type '{0}' from {2:?}"
    )]
    InvalidDataToDeserialize(String, String, Vec<u8>),

    #[error("Account {0} is requested to be deserialized but was not defined in the IDL")]
    UnknownAccount(String),

    #[error("Account with discriminator {0} is requested to be deserialized but was not defined in the IDL")]
    UnknownDiscriminatedAccount(String),

    #[error(
        "Could not find an account that matches the provided account data."
    )]
    CannotFindDeserializerForAccount,

    #[error("Account is requested to be deserialized Idl {0} version {1} has no accounts")]
    IdlHasNoAccountsAndCannotDeserializeAccountData(String, String),

    #[error("Account is requested to be via discriminator bytes but Idl {0} version {1} has no such accounts")]
    IdlHasNoAccountsDiscriminatedByDiscriminatorBytes(String, String),

    #[error("Type {0} is referenced but was not defined in the IDL")]
    CannotFindDefinedType(String),

    #[error("Variant with discriminant {0} does not exist")]
    InvalidEnumVariantDiscriminator(u8),

    #[error("Unable to parse JSON")]
    ParseJsonError(#[from] serde_json::Error),

    #[cfg(feature = "bson")]
    #[error("Raw Bson Error")]
    RawBsonError(#[from] bson::raw::Error),

    #[error("No IDL was added for the program {0}.")]
    CannotFindAccountDeserializerForProgramId(String),

    #[error("Unable to derive pubkey for the IDL to fetch")]
    IdlPubkeyError(#[from] solana_sdk::pubkey::PubkeyError),

    #[error("Unable to inflate IDl data ({0})")]
    IdlContainerShouldContainZlibData(String),

    #[error(
        "Failed to parse pubkey of the program to add IDL for '{0}' ({1})"
    )]
    FailedToParseIdlProgramPubkey(String, String),

    #[error(
        "Cannot parse account data with {0} bytes since the discriminator is at least {1} bytes"
    )]
    AccountDataTooShortForDiscriminatorBytes(usize, usize),
}
