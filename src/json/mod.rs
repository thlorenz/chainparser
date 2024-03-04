mod discriminator;
mod json_accounts_deserializer;
mod json_common;
mod json_idl_enum_variant_de;
mod json_idl_field_de;
mod json_idl_type_de;
mod json_idl_type_def_de;
mod json_serialization_opts;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use discriminator::PrefixDiscriminator;
pub use json_accounts_deserializer::JsonAccountsDeserializer;
pub use json_idl_type_def_de::JsonIdlTypeDefinitionDeserializer;
pub use json_serialization_opts::JsonSerializationOpts;

pub type JsonTypeDefinitionDeserializerMap<'opts> =
    Arc<Mutex<HashMap<String, JsonIdlTypeDefinitionDeserializer<'opts>>>>;
