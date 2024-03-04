use std::{
    collections::HashMap,
    fmt::Write,
    sync::{Arc, Mutex},
};

use solana_idl::{Idl, IdlTypeDefinitionTy};

use super::{
    discriminator::JsonAccountsDiscriminator, JsonTypeDefinitionDeserializerMap,
};
use crate::{
    deserializer::DeserializeProvider,
    discriminator::discriminator_from_data,
    errors::ChainparserResult,
    idl::IdlProvider,
    json::{JsonIdlTypeDefinitionDeserializer, JsonSerializationOpts},
};

/// Setup to  deserialize accounts for a given program. The accounts are expected to have been
/// serialized using the [borsh] format.
///
/// Uses deserializers defined inside [deserializer] modules under the hood in order to resolve the
/// appropriate [borsh] deserializers for each field.
pub struct JsonAccountsDeserializer<'opts> {
    /// Used to provide the deserializer for each account by discriminating/matching its data.
    pub discriminator: JsonAccountsDiscriminator<'opts>,

    /// The [JsonSerializationOpts] specifying how specific data types should be deserialized.
    pub serialization_opts: &'opts JsonSerializationOpts,

    /// Map of [JsonIdlTypeDefinitionDeserializer] for each type defined in the IDL.
    pub type_de_map: JsonTypeDefinitionDeserializerMap<'opts>,
}

impl<'opts> JsonAccountsDeserializer<'opts> {
    /// Tries to create an [AccounbtDeserializer] by parsing the [Idl].
    /// Fails if the IDL could not be parsed.
    ///
    /// - [json} the IDL definition in JSON format
    /// - [provider] the provider used to create the IDL
    /// - [serialization_opts] specifying how specific data types should be deserialized.
    pub fn try_from_idl(
        json: &str,
        provider: IdlProvider,
        serialization_opts: &'opts JsonSerializationOpts,
    ) -> ChainparserResult<Self> {
        let idl: Idl = serde_json::from_str(json)?;
        let de_resolver = DeserializeProvider::try_from(&idl)?;
        Ok(Self::from_idl(
            &idl,
            de_resolver,
            provider,
            serialization_opts,
        ))
    }

    /// Creates an [AccounbtDeserializer] from the provided [Idl]
    /// Fails if the IDL could not be parsed.
    ///
    /// - [idl} the IDL definition
    /// - [de_provider] to be used to deserialize each account, i.e. Borsh
    /// - [provider] the provider used to create the IDL
    /// - [serialization_opts] specifying how specific data types should be deserialized.
    pub fn from_idl(
        idl: &Idl,
        de_provider: DeserializeProvider,
        provider: IdlProvider,
        serialization_opts: &'opts JsonSerializationOpts,
    ) -> Self {
        let type_de_map = Arc::new(Mutex::new(HashMap::new()));
        let mut type_map = HashMap::<String, &IdlTypeDefinitionTy>::new();

        for type_definition in &idl.types {
            type_map.insert(type_definition.name.clone(), &type_definition.ty);
            let instance = JsonIdlTypeDefinitionDeserializer::new(
                type_definition,
                type_de_map.clone(),
                serialization_opts,
            );
            type_de_map
                .lock()
                .unwrap()
                .insert(instance.name.clone(), instance);
        }

        let discriminator = JsonAccountsDiscriminator::new(
            de_provider,
            provider,
            idl,
            &type_map,
            type_de_map.clone(),
            serialization_opts,
        );

        Self {
            serialization_opts,
            discriminator,
            type_de_map,
        }
    }

    /// Deserializes an account from the provided data.
    pub fn deserialize_account_data<W: Write>(
        &self,
        account_data: &mut &[u8],
        f: &mut W,
    ) -> ChainparserResult<()> {
        use JsonAccountsDiscriminator::*;
        match &self.discriminator {
            PrefixDiscriminator(disc) => {
                disc.deserialize_account_data(account_data, f)
            }
            MatchDiscriminator(disc) => {
                disc.deserialize_account_data(account_data, f)
            }
        }
    }

    /// Deserializes an account from the provided data.
    ///
    /// This method expects account data to **not** be prefixed with 8 bytes of discriminator data.
    /// Instead it derives that discriminator from the provided account name and then looks up the
    /// json.
    pub fn deserialize_account_data_by_name<W: Write>(
        &self,
        account_data: &mut &[u8],
        account_name: &str,
        f: &mut W,
    ) -> ChainparserResult<()> {
        use JsonAccountsDiscriminator::*;
        match &self.discriminator {
            PrefixDiscriminator(disc) => disc.deserialize_account_data_by_name(
                account_data,
                account_name,
                f,
            ),
            MatchDiscriminator(disc) => disc.deserialize_account_data_by_name(
                account_data,
                account_name,
                f,
            ),
        }
    }

    /// Resolves the account name for the provided account data.
    pub fn account_name(&self, account_data: &[u8]) -> Option<&str> {
        use JsonAccountsDiscriminator::*;
        match &self.discriminator {
            PrefixDiscriminator(disc) => {
                if account_data.len() < 8 {
                    return None;
                }
                let discriminator =
                    discriminator_from_data(&account_data[0..8]);
                disc.account_name(&discriminator)
            }
            MatchDiscriminator(disc) => disc.account_name(account_data),
        }
    }
}

// The [type_de_map] can hold circular references and thus leaks memory if not cleared.
impl Drop for JsonAccountsDeserializer<'_> {
    fn drop(&mut self) {
        self.type_de_map.lock().unwrap().clear();
    }
}
