use std::fmt::Write;

use borsh::BorshDeserialize;
use solana_idl::{IdlTypeDefinition, IdlTypeDefinitionTy};

use super::{
    json_common::deserialize_fields_to_object,
    json_idl_enum_variant_de::JsonIdlEnumVariantDeserializer,
    json_idl_field_de::JsonIdlFieldDeserializer,
    JsonTypeDefinitionDeserializerMap,
};
use crate::{
    deserializer::ChainparserDeserialize,
    errors::{ChainparserError, ChainparserResult},
    json::json_serialization_opts::JsonSerializationOpts,
};

#[derive(Clone)]
pub struct JsonIdlTypeDefinitionDeserializer<'opts> {
    pub name: String,
    pub fields: Option<Vec<JsonIdlFieldDeserializer<'opts>>>,
    pub variants: Option<Vec<JsonIdlEnumVariantDeserializer<'opts>>>,
    pub type_map: JsonTypeDefinitionDeserializerMap<'opts>,
}

impl<'opts> JsonIdlTypeDefinitionDeserializer<'opts> {
    pub fn new(
        definition: &IdlTypeDefinition,
        type_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        match &definition.ty {
            IdlTypeDefinitionTy::Struct { fields } => {
                let fields = fields
                    .iter()
                    .map(|f| {
                        JsonIdlFieldDeserializer::new(f, type_map.clone(), opts)
                    })
                    .collect();
                Self {
                    name: definition.name.clone(),
                    fields: Some(fields),
                    variants: None,
                    type_map,
                }
            }
            IdlTypeDefinitionTy::Enum { variants } => {
                let variants = variants
                    .iter()
                    .map(|v| {
                        JsonIdlEnumVariantDeserializer::new(
                            v,
                            type_map.clone(),
                            opts,
                        )
                    })
                    .collect();
                Self {
                    name: definition.name.clone(),
                    fields: None,
                    variants: Some(variants),
                    type_map,
                }
            }
        }
    }

    pub fn deserialize<W: Write>(
        &self,
        de: &impl ChainparserDeserialize,
        f: &mut W,
        buf: &mut &[u8],
    ) -> ChainparserResult<()> {
        if let Some(fields) = &self.fields {
            // Struct
            deserialize_fields_to_object(de, f, buf, fields).map_err(|e| {
                ChainparserError::StructDeserializeError(
                    self.name.to_string(),
                    Box::new(e),
                )
            })
        } else {
            // Enum
            let variants = self
                .variants
                .as_ref()
                .expect("Should either have struct fields or enum variants");

            // NOTE: not handling enums whose variants start at non-zero discriminant
            // if shank/anchor ever supports that, we'll need to handle it here
            let discriminator = u8::deserialize(buf)?;
            match &variants.get(discriminator as usize) {
                Some(deser) => deser.deserialize(de, f, buf),
                None => {
                    Err(ChainparserError::InvalidEnumVariantDiscriminator(
                        discriminator,
                    ))?
                }
            }
            .map_err(|e| {
                ChainparserError::EnumDeserializeError(
                    self.name.to_string(),
                    Box::new(e),
                )
            })
        }
    }
}
