use std::fmt::Write;

use solana_idl::{EnumFields, IdlEnumVariant, IdlType};

use super::{
    json_common::{deserialize_fields_to_object, write_quoted},
    json_idl_field_de::JsonIdlFieldDeserializer,
    json_idl_type_de::JsonIdlTypeDeserializer,
    JsonTypeDefinitionDeserializerMap,
};
use crate::{
    deserializer::ChainparserDeserialize,
    errors::{ChainparserError, ChainparserResult},
    json::json_serialization_opts::JsonSerializationOpts,
};

/// Deserializes an enum variant.
/// It is very similar to the [IdlTypeDefinitionDeserializer] since non-scalar enum variants get
/// treated like a named type with fields.
/// However it handles scalar variants as well as unnamed field variants in a specific way.
#[derive(Clone)]
pub struct JsonIdlEnumVariantDeserializer<'opts> {
    pub name: String,
    pub named_fields: Option<Vec<JsonIdlFieldDeserializer<'opts>>>,
    pub tuple_types: Option<(JsonIdlTypeDeserializer<'opts>, IdlType)>,
    pub type_map: JsonTypeDefinitionDeserializerMap<'opts>,
}

impl<'opts> JsonIdlEnumVariantDeserializer<'opts> {
    pub fn new(
        variant: &IdlEnumVariant,
        type_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        let name = variant.name.clone();
        use EnumFields::*;
        match &variant.fields {
            Some(Named(fields)) => {
                let named_fields = fields
                    .iter()
                    .map(|f| {
                        JsonIdlFieldDeserializer::new(f, type_map.clone(), opts)
                    })
                    .collect();
                Self {
                    name,
                    named_fields: Some(named_fields),
                    tuple_types: None,
                    type_map,
                }
            }
            Some(Tuple(types)) => {
                let tuple_ty_de =
                    JsonIdlTypeDeserializer::new(type_map.clone(), opts);
                Self {
                    name,
                    named_fields: None,
                    tuple_types: Some((
                        tuple_ty_de,
                        IdlType::Tuple(types.clone()),
                    )),
                    type_map,
                }
            }
            None => Self {
                name,
                named_fields: None,
                tuple_types: None,
                type_map,
            },
        }
    }
    /// Deserializes the enum variant into JSON that has the same format that [serde_json] uses.
    /// This means that non-scalar variants field values are wrapped in an object whose key is the
    /// variant name.
    /// Scalar variants are just a string of the variant name.
    pub fn deserialize<W: Write>(
        &self,
        de: &impl ChainparserDeserialize,
        f: &mut W,
        buf: &mut &[u8],
    ) -> ChainparserResult<()> {
        if let Some(named_fields) = &self.named_fields {
            f.write_char('{')?;
            {
                self.write_key(f)?;
                deserialize_fields_to_object(de, f, buf, named_fields)
                    .map_err(|e| {
                        ChainparserError::EnumVariantDeserializeError(
                            self.name.to_string(),
                            Box::new(e),
                        )
                    })?;
            }
            f.write_char('}')?;
        } else if let Some((tuple_ty_de, ty)) = &self.tuple_types {
            f.write_char('{')?;
            {
                self.write_key(f)?;
                self.deserialize_tuple_fields(de, f, buf, tuple_ty_de, ty)
                    .map_err(|e| {
                        ChainparserError::EnumVariantDeserializeError(
                            self.name.to_string(),
                            Box::new(e),
                        )
                    })?;
            }
            f.write_char('}')?;
        } else {
            write_quoted(f, &self.name)?;
        }
        Ok(())
    }

    fn deserialize_tuple_fields<W: Write>(
        &self,
        de: &impl ChainparserDeserialize,
        f: &mut W,
        buf: &mut &[u8],
        tuple_el_de: &JsonIdlTypeDeserializer<'opts>,
        ty: &IdlType,
    ) -> ChainparserResult<()> {
        tuple_el_de.deserialize(de, ty, f, buf)
    }

    fn write_key<W: Write>(&self, f: &mut W) -> ChainparserResult<()> {
        f.write_char('"')?;
        f.write_str(&self.name)?;
        f.write_str("\":")?;
        Ok(())
    }
}
