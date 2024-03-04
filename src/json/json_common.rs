use std::fmt::Write;

use super::json_idl_field_de::JsonIdlFieldDeserializer;
use crate::{deserializer::ChainparserDeserialize, errors::ChainparserResult};

pub fn deserialize_fields_to_object<W: Write>(
    de: &impl ChainparserDeserialize,
    f: &mut W,
    buf: &mut &[u8],
    fields: &[JsonIdlFieldDeserializer<'_>],
) -> ChainparserResult<()> {
    f.write_char('{')?;

    for (i, field_de) in fields.iter().enumerate() {
        field_de.deserialize(de, f, buf)?;
        if (i + 1) < fields.len() {
            f.write_char(',')?;
        }
    }

    f.write_char('}')?;

    Ok(())
}

#[inline(always)]
pub fn write_quoted<W: Write>(
    f: &mut W,
    s: &str,
) -> Result<(), std::fmt::Error> {
    f.write_str("\"")?;
    f.write_str(s)?;
    f.write_str("\"")
}
