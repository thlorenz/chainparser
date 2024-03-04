use std::fmt::Write;

use solana_idl::IdlType;

use super::{json_common::write_quoted, JsonTypeDefinitionDeserializerMap};
use crate::{
    deserializer::ChainparserDeserialize,
    errors::{ChainparserError, ChainparserResult},
    json::json_serialization_opts::JsonSerializationOpts,
};

#[derive(Clone)]
pub struct JsonIdlTypeDeserializer<'opts> {
    pub type_map: JsonTypeDefinitionDeserializerMap<'opts>,
    pub opts: &'opts JsonSerializationOpts,
}

impl<'opts> JsonIdlTypeDeserializer<'opts> {
    pub fn new(
        type_map: JsonTypeDefinitionDeserializerMap<'opts>,
        opts: &'opts JsonSerializationOpts,
    ) -> Self {
        Self { type_map, opts }
    }

    pub fn deserialize<W: Write>(
        &self,
        de: &impl ChainparserDeserialize,
        ty: &IdlType,
        f: &mut W,
        buf: &mut &[u8],
    ) -> ChainparserResult<()> {
        use IdlType::{
            Bool, F32, F64, I128, I16, I32, I64, I8, U128, U16, U32, U64, U8,
        };
        match ty {
            U8 => f.write_str(&de.u8(buf)?.to_string()),
            U16 => f.write_str(&de.u16(buf)?.to_string()),
            U32 => f.write_str(&de.u32(buf)?.to_string()),
            U64 if self.opts.n64_as_string => {
                write_quoted(f, &de.u64(buf)?.to_string())
            }
            U64 => f.write_str(&de.u64(buf)?.to_string()),

            U128 if self.opts.n128_as_string => {
                write_quoted(f, &de.u128(buf)?.to_string())
            }
            U128 => f.write_str(&de.u128(buf)?.to_string()),

            I8 => f.write_str(&de.i8(buf)?.to_string()),
            I16 => f.write_str(&de.i16(buf)?.to_string()),
            I32 => f.write_str(&de.i32(buf)?.to_string()),

            I64 if self.opts.n64_as_string => {
                write_quoted(f, &de.i64(buf)?.to_string())
            }
            I64 => f.write_str(&de.i64(buf)?.to_string()),

            I128 if self.opts.n128_as_string => {
                write_quoted(f, &de.i128(buf)?.to_string())
            }
            I128 => f.write_str(&de.i128(buf)?.to_string()),

            F32 => f.write_str(&de.f32(buf)?.to_string()),
            F64 => f.write_str(&de.f64(buf)?.to_string()),

            Bool => f.write_str(&de.bool(buf)?.to_string()),

            IdlType::String => write_quoted(f, &de.string(buf)?),

            // Composites
            IdlType::Tuple(inners) => {
                let len = inners.len();
                f.write_char('[')?;
                for (i, inner) in inners.iter().enumerate() {
                    self.deserialize(de, inner, f, buf)?;
                    if i < len - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_char(']')
            }
            IdlType::Array(inner, len) => {
                f.write_char('[')?;
                for i in 0..*len {
                    self.deserialize(de, inner, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            format!("Array[{i}] size({len})"),
                            Box::new(e),
                        )
                    })?;
                    if i < len - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_char(']')
            }
            IdlType::Vec(inner) => {
                let len = de.u32(buf)?;
                f.write_char('[')?;
                for i in 0..len {
                    self.deserialize(de, inner, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            format!("Vec[{i}] size({len})"),
                            Box::new(e),
                        )
                    })?;
                    if i < len - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_char(']')
            }
            IdlType::HashMap(inner1, inner2)
            | IdlType::BTreeMap(inner1, inner2) => {
                let len = de.u32(buf)?;
                f.write_char('{')?;
                for i in 0..len {
                    f.write_char('"')?;
                    self.deserialize(de, inner1, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            format!("Key HashMap[{i}] size({len})"),
                            Box::new(e),
                        )
                    })?;
                    f.write_str("\": ")?;
                    self.deserialize(de, inner2, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            format!("Val HashMap[{i}] size({len})"),
                            Box::new(e),
                        )
                    })?;
                    if i < len - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_char('}')
            }
            IdlType::HashSet(inner) | IdlType::BTreeSet(inner) => {
                let len = de.u32(buf)?;
                f.write_char('[')?;
                for i in 0..len {
                    self.deserialize(de, inner, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            format!("HashSet[{i}] size({len})"),
                            Box::new(e),
                        )
                    })?;
                    if i < len - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_char(']')
            }
            IdlType::Option(inner) => {
                if de.option(buf)? {
                    self.deserialize(de, inner, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            "Option".to_string(),
                            Box::new(e),
                        )
                    })?;
                } else {
                    f.write_str("null")?;
                }
                Ok(())
            }
            IdlType::COption(inner) => {
                if de.coption(buf, inner)? {
                    self.deserialize(de, inner, f, buf).map_err(|e| {
                        ChainparserError::CompositeDeserializeError(
                            "Option".to_string(),
                            Box::new(e),
                        )
                    })?;
                } else {
                    f.write_str("null")?;
                }
                Ok(())
            }
            IdlType::Bytes => {
                // Bytes is the same as a u8 array, thus stringify to an array of numbers
                // in order to be able to later JSON.parse it back into a bytes array.
                f.write_char('[')?;
                let bytes = de
                    .bytes(buf)?
                    .into_iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                f.write_str(&bytes)?;
                f.write_char(']')
            }
            IdlType::PublicKey => {
                let pubkey = de.pubkey(buf)?;
                if self.opts.pubkey_as_base58 {
                    write_quoted(f, &pubkey.to_string())?;
                } else {
                    write!(f, "{:?}", pubkey.to_bytes())?;
                }
                Ok(())
            }
            IdlType::Defined(name) => {
                let ty = { self.type_map.lock().unwrap().get(name).cloned() };
                match ty {
                    Some(deser) => {
                        deser.deserialize(de, f, buf).map_err(|e| {
                            ChainparserError::CompositeDeserializeError(
                                format!("Defined('{name}')"),
                                Box::new(e),
                            )
                        })?;
                        Ok(())
                    }
                    None => Err(ChainparserError::CannotFindDefinedType(
                        name.to_string(),
                    ))?,
                }
            }
        }?;
        Ok(())
    }
}
