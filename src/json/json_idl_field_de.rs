use std::fmt::Write;

use solana_idl::{IdlField, IdlType};

use super::{
    json_idl_type_de::JsonIdlTypeDeserializer,
    JsonTypeDefinitionDeserializerMap,
};
use crate::{
    deserializer::ChainparserDeserialize,
    errors::{ChainparserError, ChainparserResult},
    json::json_serialization_opts::JsonSerializationOpts,
};

#[derive(Clone)]
pub struct JsonIdlFieldDeserializer<'opts> {
    pub name: String,
    pub ty: IdlType,
    pub ty_deserealizer: JsonIdlTypeDeserializer<'opts>,
    pub type_map: JsonTypeDefinitionDeserializerMap<'opts>,
}

impl<'opts> JsonIdlFieldDeserializer<'opts> {
    pub fn new(
        field: &IdlField,
        type_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        let ty_deserealizer =
            JsonIdlTypeDeserializer::new(type_map.clone(), opts);
        Self {
            name: field.name.clone(),
            ty: field.ty.clone(),
            ty_deserealizer,
            type_map,
        }
    }

    pub fn deserialize<W: Write>(
        &self,
        de: &impl ChainparserDeserialize,
        f: &mut W,
        buf: &mut &[u8],
    ) -> ChainparserResult<()> {
        f.write_char('"')?;
        f.write_str(&self.name)?;
        f.write_str("\":")?;
        self.ty_deserealizer
            .deserialize(de, &self.ty, f, buf)
            .map_err(|e| {
                ChainparserError::FieldDeserializeError(
                    self.name.to_string(),
                    Box::new(e),
                )
            })
    }
}
