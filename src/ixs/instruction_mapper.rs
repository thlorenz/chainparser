use lazy_static::lazy_static;
use log::*;
use std::{collections::HashMap, str::FromStr};

use solana_idl::{Idl, IdlInstruction};
use solana_sdk::pubkey::Pubkey;

use super::{discriminator::discriminator_from_ix, ParseableInstruction};

#[rustfmt::skip]
lazy_static! {
    pub static ref BUILTIN_PROGRAMS: HashMap<Pubkey, &'static str> = [
        ("System Program"                , "11111111111111111111111111111111")           ,
        ("BPF Upgradeable Loader"        , "BPFLoaderUpgradeab1e11111111111111111111111"),
        ("BPF Loader 2"                  , "BPFLoader2111111111111111111111111111111111"),
        ("Config Program"                , "Config1111111111111111111111111111111111111"),
        ("Feature Program"               , "Feature111111111111111111111111111111111111"),
        ("Native Loader"                 , "NativeLoader1111111111111111111111111111111"),
        ("Stake Program"                 , "Stake11111111111111111111111111111111111111"),
        ("Sysvar"                        , "Sysvar1111111111111111111111111111111111111"),
        ("Vote Program"                  , "Vote111111111111111111111111111111111111111"),
        ("Stake Config"                  , "StakeConfig11111111111111111111111111111111"),
        ("Sol Program"                   , "So11111111111111111111111111111111111111112"),
        ("Clock Sysvar"                  , "SysvarC1ock11111111111111111111111111111111"),
        ("Epoch Schedule Sysvar"         , "SysvarEpochSchedu1e111111111111111111111111"),
        ("Fees Sysvar"                   , "SysvarFees111111111111111111111111111111111"),
        ("Last Restart Slog Sysvar"      , "SysvarLastRestartS1ot1111111111111111111111"),
        ("Recent Blockhashes Sysvar"     , "SysvarRecentB1ockHashes11111111111111111111"),
        ("Rent Sysvar"                   , "SysvarRent111111111111111111111111111111111"),
        ("Slot Hashes"                   , "SysvarS1otHashes111111111111111111111111111"),
        ("Slot History"                  , "SysvarS1otHistory11111111111111111111111111"),
        ("Stake History"                 , "SysvarStakeHistory1111111111111111111111111"),
        ("MagicBlock System Program"     , "Magic11111111111111111111111111111111111111"),
        ("MagicBlock Delegation Program" , "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh"),
        ("Luzid Authority"               , "LUzidNSiPNjYNkxZcUm5hYHwnWPwsUfh2US1cpWwaBm"),
    ]
    .into_iter()
    .map(|(name, key)| (Pubkey::from_str(key).unwrap(), name))
    .collect();
}

pub fn map_instruction(
    instruction: &impl ParseableInstruction,
    idl: Option<&Idl>,
) -> InstructionMapResult {
    InstructionMapper::map_accounts(instruction, idl)
}

pub struct InstructionMapper {
    idl_instruction: IdlInstruction,
}

pub struct InstructionMapResult {
    pub accounts: HashMap<Pubkey, String>,
    pub instruction_name: Option<String>,
    pub program_name: Option<String>,
}

impl InstructionMapper {
    /// First determines which IDL to use via the [program_id] of the instruction.
    /// Then it finds the best matching IDL instruction for provided instruction and
    /// creates an entry for each account pubkey providing its name.
    pub fn map_accounts(
        instruction: &impl ParseableInstruction,
        idl: Option<&Idl>,
    ) -> InstructionMapResult {
        let mapper = idl
            .as_ref()
            .and_then(|idl| Self::determine_accounts_mapper(instruction, idl));
        let program_name = idl.as_ref().map(|idl| idl.name.to_string());
        let program_id = instruction.program_id();

        let mut accounts = HashMap::new();
        let mut instruction_name = None::<String>;
        let ix_accounts = instruction.accounts();
        for (idx, pubkey) in ix_accounts.into_iter().enumerate() {
            if let Some(name) = BUILTIN_PROGRAMS.get(&pubkey) {
                accounts.insert(pubkey, name.to_string());
                continue;
            }
            if let Some(program_name) = program_name.as_ref() {
                if &pubkey == program_id {
                    accounts.insert(pubkey, program_name.to_string());
                    continue;
                }
            }
            if let Some(mapper) = &mapper {
                let name = mapper
                    .idl_instruction
                    .accounts
                    .get(idx)
                    .map(|x| x.name().to_string());
                if let Some(name) = name {
                    accounts.insert(pubkey, name);
                }
                instruction_name
                    .replace(mapper.idl_instruction.name.to_string());
            }
        }
        let program_name = idl.map(|x| x.name.to_string()).or_else(|| {
            BUILTIN_PROGRAMS.get(program_id).map(|x| x.to_string())
        });

        InstructionMapResult {
            accounts,
            instruction_name,
            program_name,
        }
    }

    fn determine_accounts_mapper(
        instruction: &impl ParseableInstruction,
        idl: &Idl,
    ) -> Option<InstructionMapper> {
        find_best_matching_idl_ix(&idl.instructions, instruction)
            .map(|idl_instruction| InstructionMapper { idl_instruction })
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
        if disc.len() > ix.data().len() {
            continue;
        }
        let mut score = 0;
        for (a, b) in disc.iter().zip(ix.data()) {
            if a != b {
                break;
            }
            score += 1;
        }
        if score > best_match_score {
            best_match = Some(idl_ix);
            best_match_score = score;
        }
    }
    best_match.cloned()
}
