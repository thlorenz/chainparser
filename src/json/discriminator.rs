use std::{collections::HashMap, fmt::Write};

use solana_idl::{Idl, IdlTypeDefinition, IdlTypeDefinitionTy};

use crate::{
    deserializer::DeserializeProvider,
    discriminator::{
        account_discriminator, match_discriminator::MatchDiscriminators,
        DiscriminatorBytes,
    },
    errors::{ChainparserError, ChainparserResult},
    idl::IdlProvider,
    json::{
        JsonIdlTypeDefinitionDeserializer, JsonSerializationOpts,
        JsonTypeDefinitionDeserializerMap,
    },
};

// -----------------
// PrefixDiscriminator
// -----------------

/// This is the common way of resolving the account type for account data.
/// It expects the first 8 bytes of data to hold the account discriminator as is the case for
/// anchor accounts.
/// This is what is used for Anchor accounts.
pub struct PrefixDiscriminator<'opts> {
    /// Allows looking up a account names by discriminator.
    account_names: HashMap<DiscriminatorBytes, String>,

    /// The deserializers for accounts of this program keyed by the discriminator of each account
    /// type.
    deserializers:
        HashMap<DiscriminatorBytes, JsonIdlTypeDefinitionDeserializer<'opts>>,

    de_provider: DeserializeProvider,
}

impl<'opts> PrefixDiscriminator<'opts> {
    pub fn new(
        de_provider: DeserializeProvider,
        accounts: &[IdlTypeDefinition],
        type_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        let mut by_name = HashMap::<String, DiscriminatorBytes>::new();
        let mut deserializers = HashMap::<
            DiscriminatorBytes,
            JsonIdlTypeDefinitionDeserializer<'opts>,
        >::new();

        for account_definition in accounts {
            let type_deserializer =
                JsonIdlTypeDefinitionDeserializer::<'opts>::new(
                    account_definition,
                    type_map.clone(),
                    opts,
                );

            // NOTE: for now we assume that one account doesn't reference another
            //       thus we don't include it in the lookup map for nested types
            //       Similarly for instruction args once we support them
            let discriminator = account_discriminator(&account_definition.name);
            deserializers.insert(discriminator, type_deserializer);
            by_name.insert(account_definition.name.clone(), discriminator);
        }

        let account_names = by_name
            .iter()
            .map(|(name, discriminator)| (*discriminator, name.clone()))
            .collect();

        Self {
            de_provider,
            account_names,
            deserializers,
        }
    }

    /// Deserializes
    pub fn deserialize_account_data<W: Write>(
        &self,
        account_data: &mut &[u8],
        f: &mut W,
    ) -> ChainparserResult<()> {
        if account_data.len() < 8 {
            return Err(
                ChainparserError::AccountDataTooShortForDiscriminatorBytes(
                    account_data.len(),
                    8,
                ),
            );
        }
        let discriminator = &account_data[..8];
        let deserializer =
            self.deserializers.get(discriminator).ok_or_else(|| {
                ChainparserError::UnknownDiscriminatedAccount(format!(
                    "disciminator: {discriminator:?}"
                ))
            })?;

        let data = &mut &account_data[8..];
        deserialize(&self.de_provider, deserializer, f, data)
    }

    pub fn deserialize_account_data_by_name<W: Write>(
        &self,
        account_data: &mut &[u8],
        account_name: &str,
        f: &mut W,
    ) -> ChainparserResult<()> {
        let discriminator = account_discriminator(account_name);
        let deserializer =
            self.deserializers.get(&discriminator).ok_or_else(|| {
                ChainparserError::UnknownAccount(account_name.to_string())
            })?;

        deserialize(&self.de_provider, deserializer, f, account_data)
    }

    pub fn account_name(
        &self,
        discriminator: &DiscriminatorBytes,
    ) -> Option<&str> {
        self.account_names.get(discriminator).map(|s| s.as_str())
    }
}

// -----------------
// MatchDiscriminator
// -----------------

/// This discriminator is used when no discriminator bytes are added to the account data.
/// It tries as best as possible to match account data to the account type via size and expected
/// patterns in the data.
pub struct MatchDiscriminator<'opts> {
    /// Used to match the shape of the account against a given buffer to identify it and provide
    /// its name.
    discriminators: MatchDiscriminators,

    /// The deserializers for accounts of this program keyed by the name of each account
    /// type.
    deserializer_by_name:
        HashMap<String, JsonIdlTypeDefinitionDeserializer<'opts>>,

    de_provider: DeserializeProvider,
}

impl<'opts> MatchDiscriminator<'opts> {
    pub fn new(
        de_provider: DeserializeProvider,
        accounts: &[IdlTypeDefinition],
        type_map: &HashMap<String, &IdlTypeDefinitionTy>,
        type_de_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        let discriminators = MatchDiscriminators::from((accounts, type_map));
        let mut deserializer_by_name =
            HashMap::<String, JsonIdlTypeDefinitionDeserializer<'opts>>::new();

        for disc in discriminators.iter() {
            let deserializer = JsonIdlTypeDefinitionDeserializer::<'opts>::new(
                &disc.account,
                type_de_map.clone(),
                opts,
            );
            deserializer_by_name
                .insert(disc.account_name().to_string(), deserializer);
        }
        Self {
            de_provider,
            discriminators,
            deserializer_by_name,
        }
    }

    pub fn deserialize_account_data<W: Write>(
        &self,
        account_data: &mut &[u8],
        f: &mut W,
    ) -> ChainparserResult<()> {
        if account_data.is_empty() {
            return Err(
                ChainparserError::AccountDataTooShortForDiscriminatorBytes(
                    0, 1,
                ),
            );
        }
        match self.discriminators.find_match_name(account_data) {
            Some(name) => {
                self.deserialize_account_data_by_name(account_data, name, f)
            }
            None => Err(ChainparserError::CannotFindDeserializerForAccount),
        }
    }

    pub fn deserialize_account_data_by_name<W: Write>(
        &self,
        account_data: &mut &[u8],
        account_name: &str,
        f: &mut W,
    ) -> ChainparserResult<()> {
        match self.deserializer_by_name.get(account_name) {
            Some(deserializer) => {
                deserialize(&self.de_provider, deserializer, f, account_data)
            }
            None => {
                Err(ChainparserError::UnknownAccount(account_name.to_string()))
            }
        }
    }

    pub fn account_name(&self, account_data: &[u8]) -> Option<&str> {
        self.discriminators.find_match_name(account_data)
    }
}

// -----------------
// JsonAccountsDiscriminator
// -----------------
pub enum JsonAccountsDiscriminator<'opts> {
    PrefixDiscriminator(PrefixDiscriminator<'opts>),
    MatchDiscriminator(MatchDiscriminator<'opts>),
}

impl<'opts> JsonAccountsDiscriminator<'opts> {
    pub fn new(
        de_provider: DeserializeProvider,
        provider: IdlProvider,
        idl: &Idl,
        type_map: &HashMap<String, &IdlTypeDefinitionTy>,
        type_de_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        match provider {
            IdlProvider::Anchor => {
                Self::PrefixDiscriminator(PrefixDiscriminator::new(
                    de_provider,
                    &idl.accounts,
                    type_de_map,
                    opts,
                ))
            }
            _ => Self::MatchDiscriminator(MatchDiscriminator::new(
                de_provider,
                &idl.accounts,
                type_map,
                type_de_map,
                opts,
            )),
        }
    }
}

// -----------------
// Helpers
// -----------------
fn deserialize(
    de_provider: &DeserializeProvider,
    deserializer: &JsonIdlTypeDefinitionDeserializer,
    f: &mut impl Write,
    data: &mut &[u8],
) -> ChainparserResult<()> {
    match de_provider {
        DeserializeProvider::Borsh(de) => deserializer.deserialize(de, f, data),
        DeserializeProvider::Spl(de) => deserializer.deserialize(de, f, data),
    }
}
