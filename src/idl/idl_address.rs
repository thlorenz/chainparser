use std::collections::HashMap;

use solana_idl::{IdlType, IdlTypeDefinitionTy};
use solana_sdk::pubkey::Pubkey;

use super::IdlProvider;
use crate::errors::ChainparserResult;

const ANCHOR_SEED: &str = "anchor:idl";
const SHANK_SEED: &str = "shank:idl";

/// Resolves the address of the account where the program [IDL] is stored.
///
/// - [provider] that uploaded the [IDL]
/// - [program_id] address of the program
pub fn try_idl_address(
    provider: &IdlProvider,
    program_id: &Pubkey,
) -> ChainparserResult<Pubkey> {
    let (base, _) = Pubkey::find_program_address(&[], program_id);
    let seed = match provider {
        IdlProvider::Anchor => ANCHOR_SEED,
        IdlProvider::Shank => SHANK_SEED,
    };
    let key = Pubkey::create_with_seed(&base, seed, program_id)?;
    Ok(key)
}

/// Resolves the addresses of IDL accounts for `(anchor, shank)`.
pub fn get_idl_addresses(
    program_id: &Pubkey,
) -> (Option<Pubkey>, Option<Pubkey>) {
    let (base, _) = Pubkey::find_program_address(&[], program_id);
    let anchor = Pubkey::create_with_seed(&base, ANCHOR_SEED, program_id).ok();
    let shank = Pubkey::create_with_seed(&base, SHANK_SEED, program_id).ok();
    (anchor, shank)
}

pub fn is_idl_addess(program_id: &Pubkey, address: &Pubkey) -> bool {
    let (anchor, shank) = get_idl_addresses(program_id);
    let is_anchor_idl = matches!(anchor, Some(anchor) if anchor == *address);
    if is_anchor_idl {
        return true;
    }
    matches!(shank, Some(shank) if shank == *address)
}

pub(crate) fn idl_type_bytes(
    ty: &IdlType,
    type_map: Option<&HashMap<String, &IdlTypeDefinitionTy>>,
) -> Option<usize> {
    use IdlType::*;
    match ty {
        U8 => Some(1),
        U16 => Some(2),
        U32 => Some(4),
        U64 => Some(8),
        U128 => Some(16),
        I8 => Some(1),
        I16 => Some(2),
        I32 => Some(4),
        I64 => Some(8),
        I128 => Some(16),
        F32 => Some(4),
        F64 => Some(8),
        Bool => Some(1),
        PublicKey => Some(32),
        IdlType::Array(inner, len) => {
            idl_type_bytes(inner, type_map).map(|x| x * len)
        }
        IdlType::COption(inner) => {
            idl_type_bytes(inner, type_map).map(|x| x + 4)
        }
        Defined(s) => {
            if let Some(ty) = type_map.and_then(|map| map.get(s)) {
                idl_def_bytes(ty, type_map)
            } else {
                None
            }
        }
        // NOTE: for Option the size is different depending if it is None or Some
        _ => None,
    }
}
pub(crate) fn idl_def_bytes(
    ty: &IdlTypeDefinitionTy,
    type_map: Option<&HashMap<String, &IdlTypeDefinitionTy>>,
) -> Option<usize> {
    match ty {
        IdlTypeDefinitionTy::Struct { fields } => {
            let mut struct_size = 0;
            for field in fields {
                if let Some(size) = idl_type_bytes(&field.ty, type_map) {
                    struct_size += size;
                } else {
                    return None;
                }
            }
            Some(struct_size)
        }
        IdlTypeDefinitionTy::Enum { variants } => {
            if variants.iter().all(|variant| variant.fields.is_none()) {
                return Some(1);
            }
            // if variants have different sizes then we cannot determine the size
            // it will take without data
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    pub fn str_to_pubkey(pubkey_str: &str) -> Pubkey {
        FromStr::from_str(pubkey_str).expect("pubkey from string")
    }

    #[test]
    fn idl_address_test() {
        let program_id =
            str_to_pubkey("cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ");

        let anchor_idl_address =
            try_idl_address(&IdlProvider::Anchor, &program_id).unwrap();
        let shank_idl_address =
            try_idl_address(&IdlProvider::Shank, &program_id).unwrap();

        assert_eq!(
            anchor_idl_address.to_string(),
            "CggtNXgCye2qk7fLohonNftqaKT35GkuZJwHrRghEvSF"
        );
        assert_eq!(
            shank_idl_address.to_string(),
            "AEUhdmwzSea7oYDWhAiSBArqq6tBLFNNZZ448wfbaV3Z"
        );
    }

    #[test]
    fn get_idl_addresses_test() {
        let (anchor, shank) = get_idl_addresses(&str_to_pubkey(
            "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ",
        ));
        assert_eq!(
            anchor.unwrap().to_string(),
            "CggtNXgCye2qk7fLohonNftqaKT35GkuZJwHrRghEvSF"
        );
        assert_eq!(
            shank.unwrap().to_string(),
            "AEUhdmwzSea7oYDWhAiSBArqq6tBLFNNZZ448wfbaV3Z"
        );
    }

    #[test]
    fn is_idl_address_test() {
        let program_id =
            str_to_pubkey("cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ");
        assert!(is_idl_addess(
            &program_id,
            &str_to_pubkey("CggtNXgCye2qk7fLohonNftqaKT35GkuZJwHrRghEvSF")
        ));
        assert!(is_idl_addess(
            &program_id,
            &str_to_pubkey("AEUhdmwzSea7oYDWhAiSBArqq6tBLFNNZZ448wfbaV3Z")
        ));
        assert!(!is_idl_addess(&program_id, &Pubkey::default()));
    }
}
