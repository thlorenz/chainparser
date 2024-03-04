use std::collections::{HashMap, HashSet};
pub use std::fmt::Write;

use solana_idl::Idl;
use solana_sdk::pubkey::Pubkey;

pub use crate::json::{JsonAccountsDeserializer, JsonSerializationOpts};
use crate::{
    deserializer::DeserializeProvider,
    errors::{ChainparserError, ChainparserResult},
    idl::{try_find_idl_for_program, IdlProvider, IDL_PROVIDERS},
    traits::AccountProvider,
};

/// Setup to  deserialize accounts for a given program. The accounts are expected to have been
/// serialized using the [borsh] format.
///
/// Uses deserializers defined inside [deserializer] modules under the hood in order to resolve the
/// appropriate [borsh] deserializers for each field.
pub struct ChainparserDeserializer<'opts> {
    /// The deserializers for accounts of for each program
    json_account_deserializers:
        HashMap<String, JsonAccountsDeserializer<'opts>>,

    /// The [JsonSerializationOpts] specifying how specific data types should be deserialized.
    json_serialization_opts: &'opts JsonSerializationOpts,
}

impl<'opts> ChainparserDeserializer<'opts> {
    /// Creates an instance of a [ChainparserDeserializer].
    /// Make sure to use [ChainparserDeserializer::add_idl_json] for each program _before_ attempting
    /// to deserialize accounts for it.
    ///
    /// - [serialization_opts] specifying how specific data types should be deserialized.
    pub fn new(json_serialization_opts: &'opts JsonSerializationOpts) -> Self {
        Self {
            json_account_deserializers: HashMap::new(),
            json_serialization_opts,
        }
    }

    /// Attempts to find the IDL account for the given [program_id] and adds it to the
    /// deserializer.
    /// It first tries to find an anchor IDl account and then tries shank.
    /// Returns [Some::<IdlProvider>] if the IDL was found and added, and [None::<IdlProvider>] if
    /// neither a shank nor an anchor IDL account was found.
    pub fn try_add_idl_for_program<T: AccountProvider>(
        &mut self,
        account_provider: &T,
        program_id: &Pubkey,
    ) -> ChainparserResult<Option<IdlProvider>> {
        for idl_provider in IDL_PROVIDERS {
            if let Some(idl) = try_find_idl_for_program(
                account_provider,
                program_id,
                idl_provider,
            )? {
                self.add_idl(
                    program_id.to_string(),
                    idl,
                    idl_provider.clone(),
                )?;
                return Ok(Some(idl_provider.clone()));
            }
        }
        Ok(None)
    }

    /// Parses an [IDL] specification from the provided [idl_json] for the [id] and adds a
    /// json accounts deserializer derived from it.
    /// The id is usually the program id, possibly combined with the slot at which the IDL was
    /// uploaded.
    pub fn add_idl_json(
        &mut self,
        id: String,
        idl_json: &str,
        provider: IdlProvider,
    ) -> ChainparserResult<()> {
        let json_deserializer = JsonAccountsDeserializer::try_from_idl(
            idl_json,
            provider,
            self.json_serialization_opts,
        )?;
        self.json_account_deserializers
            .insert(id, json_deserializer);
        Ok(())
    }

    /// Adds [IDL] specification from the provided [idl] for the [id] and adds a
    /// json accounts deserializer derived from it.
    /// The id is usually the program id, possibly combined with the slot at which the IDL was
    /// uploaded.
    pub fn add_idl(
        &mut self,
        id: String,
        idl: Idl,
        provider: IdlProvider,
    ) -> ChainparserResult<()> {
        let de_provider = DeserializeProvider::try_from(&idl)?;

        let json_deserializer = JsonAccountsDeserializer::from_idl(
            &idl,
            de_provider,
            provider,
            self.json_serialization_opts,
        );
        self.json_account_deserializers
            .insert(id, json_deserializer);
        Ok(())
    }

    pub fn account_name(&self, id: &str, account_data: &[u8]) -> Option<&str> {
        self.json_account_deserializers
            .get(id)
            .and_then(|deserializer| deserializer.account_name(account_data))
    }

    /// Returns `true` if the IDL of the given [id] has been added to the deserializer.
    /// The id is usually the program id, possibly combined with the slot at which the IDL was
    /// uploaded.
    pub fn has_idl(&self, id: &str) -> bool {
        self.json_account_deserializers.contains_key(id)
    }

    /// Returns all program ids for which IDLs have been added to the deserializer.
    pub fn added_idls(&self) -> HashSet<String> {
        self.json_account_deserializers.keys().cloned().collect()
    }

    /// Deserializes an account to a JSON string.
    ///
    /// In order to specify a custom [Write] writer, i.e. a socket connection to write to, use
    /// [deserialize_account] instead.
    ///
    /// - [id] is the program id of program that owns the account, possibly combined with the slot
    /// at which the IDL to use for deserialization was uploaded.
    ///   make sure to add it's IDL before via [ChainparserDeserializer::add_idl_json].
    /// - [account_data] is the raw account data as a byte array
    pub fn deserialize_account_to_json_string(
        &self,
        id: &str,
        account_data: &mut &[u8],
    ) -> ChainparserResult<String> {
        let mut f = String::new();
        self.deserialize_account_to_json(id, account_data, &mut f)?;
        Ok(f)
    }

    /// Deserializes an account and writes the resulting JSON to the provided [Write] write [f].
    ///
    /// - [id] is the program id of program that owns the account, possibly combined with the slot
    /// at which the IDL to use for deserialization was uploaded. Make sure to add it's IDL before
    /// via [ChainparserDeserializer::add_idl_json].
    /// - [account_data] is the raw account data as a byte array
    /// - [f] is the [Write] writer to write the resulting JSON to, i.e. `std::io::stdout()` or
    /// `String::new()`
    pub fn deserialize_account_to_json<W: Write>(
        &self,
        id: &str,
        account_data: &mut &[u8],
        f: &mut W,
    ) -> ChainparserResult<()> {
        let deserializer =
            self.json_account_deserializers.get(id).ok_or_else(|| {
                ChainparserError::CannotFindAccountDeserializerForProgramId(
                    id.to_string(),
                )
            })?;

        deserializer.deserialize_account_data(account_data, f)?;
        Ok(())
    }

    pub fn deserialize_account_to_json_by_name<W: Write>(
        &self,
        id: &str,
        name: &str,
        account_data: &mut &[u8],
        f: &mut W,
    ) -> ChainparserResult<()> {
        let deserializer =
            self.json_account_deserializers.get(id).ok_or_else(|| {
                ChainparserError::CannotFindAccountDeserializerForProgramId(
                    id.to_string(),
                )
            })?;

        deserializer.deserialize_account_data_by_name(account_data, name, f)?;
        Ok(())
    }
}
