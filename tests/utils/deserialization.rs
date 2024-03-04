use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use borsh::BorshSerialize;
pub use chainparser::json::{
    JsonIdlTypeDefinitionDeserializer, JsonSerializationOpts,
};
use serde::Deserialize;
use solana_idl::{IdlField, IdlType, IdlTypeDefinition};

pub fn to_if(name: &str, ty: IdlType) -> IdlField {
    IdlField {
        name: name.to_string(),
        ty,
        attrs: None,
    }
}

pub fn process_test_case_json<'de, 'a, T>(
    label: &str,
    idl_type_defs: &[&IdlTypeDefinition],
    instance: T,
    deser_key: &str,
    writer: &'de mut String,
    opts: Option<JsonSerializationOpts>,
    buf: Option<Vec<u8>>,
) where
    T: Deserialize<'de> + BorshSerialize + std::fmt::Debug + Eq + PartialEq,
{
    let type_map = Arc::new(Mutex::new(HashMap::new()));
    let opts = opts.unwrap_or_default();

    // 1. process all idl type defs to populate the type map and then use
    for idl_type_def in idl_type_defs {
        let deser = JsonIdlTypeDefinitionDeserializer::new(
            idl_type_def,
            type_map.clone(),
            &opts,
        );
        type_map
            .lock()
            .unwrap()
            .insert(idl_type_def.name.clone(), deser);
    }

    // 2. deserialize the instance with the requested json
    let buf = buf.unwrap_or_else(|| instance.try_to_vec().unwrap());

    let deser = {
        type_map
            .lock()
            .unwrap()
            .get(deser_key)
            .cloned()
            .unwrap_or_else(|| panic!("Unable to find json {deser_key}"))
    };
    let de = chainparser::borsh::BorshDeserializer;
    deser
        .deserialize(&de, writer, &mut &buf[..])
        .expect("Failed to deserialize");

    let res = match serde_json::from_str::<T>(writer) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("JSON: {writer}");
            panic!("Failed to deserialize: {e}")
        }
    };
    if res != instance {
        eprintln!("JSON: {writer}");
    }
    assert_eq!(res, instance, "{label}");
}

pub fn process_test_case_json_compare_str(
    label: &str,
    idl_type_defs: &[&IdlTypeDefinition],
    deser_key: &str,
    writer: &mut String,
    opts: Option<JsonSerializationOpts>,
    buf: Vec<u8>,
    expected: &str,
) {
    let type_map = Arc::new(Mutex::new(HashMap::new()));
    let opts = opts.unwrap_or_default();

    // 1. process all idl type defs to populate the type map and then use
    for idl_type_def in idl_type_defs {
        let deser = JsonIdlTypeDefinitionDeserializer::new(
            idl_type_def,
            type_map.clone(),
            &opts,
        );
        type_map
            .lock()
            .unwrap()
            .insert(idl_type_def.name.clone(), deser);
    }

    let deser = {
        type_map
            .lock()
            .unwrap()
            .get(deser_key)
            .cloned()
            .unwrap_or_else(|| panic!("Unable to find json {deser_key}"))
    };
    let de = chainparser::borsh::BorshDeserializer;
    deser
        .deserialize(&de, writer, &mut &buf[..])
        .expect("Failed to deserialize");

    assert_eq!(writer, expected, "{label}");
}
