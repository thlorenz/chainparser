use heck::ToSnakeCase;
use solana_idl::IdlInstruction;
use solana_sdk::hash;

// Namespace for calculating instruction sighash signatures for any instruction
// not affecting program state.
const SIGHASH_GLOBAL_NAMESPACE: &str = "global";

pub fn discriminator_from_ix(ix: &IdlInstruction) -> Vec<u8> {
    ix.discriminant
        .as_ref()
        // Newer Anchor Versions >=0.30 add the discriminator value which
        // is moved to the `bytes` property
        // Shank adds the indes of the instruction to the `value` property
        // instead.
        .map(|x| x.bytes.clone().unwrap_or(vec![x.value]))
        // If we don't find it in either we assume it is an older anchor IDL
        // and derive the discriminator the same way that anchor did before.
        .unwrap_or_else(|| {
            anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, &ix.name).to_vec()
        })
}

/// Replicates the mechanism that anchor used in order to derive a discriminator
/// from the name of an instruction.
fn anchor_sighash(namespace: &str, ix_name: &str) -> [u8; 8] {
    // NOTE: even though the name of the ix is lower camel cased in the IDL it
    // seems that the IX discriminator is derived from the snake case version
    // (see discriminator_for_house_initialize test below which came from a real case)
    let ix_name = ix_name.to_snake_case();

    let preimage = format!("{namespace}:{ix_name}");

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);
    sighash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_for_delegate() {
        let sighash = anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, "delegate");
        assert_eq!(sighash, [90, 147, 75, 178, 85, 88, 4, 137]);
    }

    #[test]
    fn discriminator_for_increment() {
        let sighash = anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, "increment");
        assert_eq!(sighash, [11, 18, 104, 9, 104, 174, 59, 33]);
    }

    #[test]
    fn discriminator_for_add_entity() {
        let sighash = anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, "add_entity");
        assert_eq!(sighash, [163, 241, 57, 35, 244, 244, 48, 57]);
    }

    #[test]
    fn discriminator_for_process_undelegation() {
        let sighash =
            anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, "process_undelegation");
        assert_eq!(sighash, [196, 28, 41, 206, 48, 37, 51, 167]);
    }
    #[test]
    fn discriminator_for_house_initialize() {
        // 8d 53 7d 73 a2 98 51 e7 e1 5f 47 02 00 00 00 00
        let sighash =
            anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, "house_initialize");
        assert_eq!(sighash, [141, 83, 125, 115, 162, 152, 81, 231]);
        let sighash =
            anchor_sighash(SIGHASH_GLOBAL_NAMESPACE, "houseInitialize");
        assert_eq!(sighash, [141, 83, 125, 115, 162, 152, 81, 231]);
    }
}
