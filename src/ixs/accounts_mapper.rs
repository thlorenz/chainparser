use log::*;
use std::collections::HashMap;

use solana_idl::{Idl, IdlInstruction};
use solana_sdk::pubkey::Pubkey;

use crate::{
    idl::try_find_idl_and_provider_for_program, traits::AccountProvider,
    Result as ChainparserResult,
};

use super::{discriminator::discriminator_from_ix, ParseableInstruction};

pub fn map_instruction_account_labels<T: AccountProvider>(
    account_provider: &T,
    instruction: &impl ParseableInstruction,
    idl: Option<Idl>,
) -> ChainparserResult<HashMap<Pubkey, String>> {
    let idl = match idl {
        Some(idl) => idl,
        None => {
            let idl = try_find_idl_and_provider_for_program(
                account_provider,
                instruction.program_id(),
            )
            .map(|x| x.map(|(idl, _)| idl))?;
            if let Some(idl) = idl {
                idl
            } else {
                return Ok(HashMap::new());
            }
        }
    };
    Ok(InstructionAccountsMapper::map_accounts(instruction, idl))
}

pub struct InstructionAccountsMapper {
    accounts: Vec<Pubkey>,
    idl_instruction: IdlInstruction,
}

impl InstructionAccountsMapper {
    /// First determines which IDL to use via the [program_id] of the instruction.
    /// Then it finds the best matching IDL instruction for provided instruction and
    /// creates an entry for each account pubkey providing its name.
    pub fn map_accounts(
        instruction: &impl ParseableInstruction,
        idl: Idl,
    ) -> HashMap<Pubkey, String> {
        Self::determine_accounts_mapper(instruction, &idl)
            .map(|mapper| {
                let mut accounts = HashMap::new();
                for idx in 0..mapper.accounts.len() {
                    let pubkey = mapper.accounts[idx];
                    let name = mapper
                        .idl_instruction
                        .accounts
                        .get(idx)
                        .map(|x| x.name().to_string());
                    if let Some(name) = name {
                        accounts.insert(pubkey, name);
                    }
                }

                accounts
            })
            .unwrap_or_default()
    }

    fn determine_accounts_mapper(
        instruction: &impl ParseableInstruction,
        idl: &Idl,
    ) -> Option<InstructionAccountsMapper> {
        find_best_matching_idl_ix(&idl.instructions, instruction).map(
            |idl_instruction| InstructionAccountsMapper {
                accounts: instruction.accounts(),
                idl_instruction,
            },
        )
    }
}

fn find_best_matching_idl_ix(
    ix_idls: &[IdlInstruction],
    ix: &impl ParseableInstruction,
) -> Option<IdlInstruction> {
    let mut best_match = None;
    let mut best_match_score = 0;
    for idl_ix in ix_idls {
        let disc = discriminator_from_ix(idl_ix);
        trace!("Discriminator for '{}': {:?}", idl_ix.name, disc);
        let score = disc.iter().zip(ix.data()).filter(|(a, b)| a == b).count();
        if score > best_match_score {
            best_match = Some(idl_ix);
            best_match_score = score;
        }
    }
    best_match.cloned()
}