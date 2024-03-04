//! Loading IDLs from Data or JSON and convert into a format storable in the Validator
use std::{fs, str::FromStr};

use solana_sdk::{
    account::{Account, AccountSharedData},
    pubkey::Pubkey,
    stake_history::Epoch,
};

use super::{encode_idl_account_json, try_idl_address, IdlProvider};
use crate::errors::{ChainparserError, ChainparserResult};

/// Given the full path to an IDL JSON file, returns the [Pubkey] of the IDL
/// account and an [AccountSharedData] that can be loaded into the validator at
/// that address.
/// Prepares leading data following IDL account specs (anchor).
/// For more se [account_shared_data_for_idl_json_file].
pub fn idl_pubkey_and_deployable_data(
    program_id: &str,
    idl_path: &str,
) -> ChainparserResult<(Pubkey, AccountSharedData)> {
    let program_pubkey = Pubkey::from_str(program_id).map_err(|err| {
        ChainparserError::FailedToParseIdlProgramPubkey(
            program_id.to_string(),
            format!("{:#?}", err),
        )
    })?;
    let idl_pubkey = try_idl_address(&IdlProvider::Anchor, &program_pubkey)?;

    Ok((
        idl_pubkey,
        account_shared_data_for_idl_json_file(program_pubkey, idl_path)?,
    ))
}

/// Given the full path to an IDL JSON file, returns an [AccountSharedData] that
/// can be loaded into the validator.
/// Note that the data includes the program_pubkey [Pubkey] as the owner of the
/// IDL account in the data that is stored in the IDL account.
/// This is necessary for tools like the explorer to accept this as a proper
/// IDL account.
pub fn account_shared_data_for_idl_json_file(
    program_pubkey: Pubkey,
    idl_path: &str,
) -> ChainparserResult<AccountSharedData> {
    let idl_json = fs::read_to_string(idl_path)?;
    account_shared_data_for_idl_json(program_pubkey, &idl_json)
}

/// Given the [Pubkey] of a program and the JSON of an IDL, returns an
/// [AccountSharedData] that can be loaded into the validator.
pub fn account_shared_data_for_idl_json(
    program_pubkey: Pubkey,
    idl_json: &str,
) -> ChainparserResult<AccountSharedData> {
    // 1. Encode into zip
    let encoded_idl_data = encode_idl_account_json(&program_pubkey, idl_json)?;

    // 2. obtain account shared data from it
    let account_shared_data =
        account_shared_data_for_idl(program_pubkey, encoded_idl_data)?;
    Ok(account_shared_data)
}

fn account_shared_data_for_idl(
    program_pubkey: Pubkey,
    encoded_idl_data: Vec<u8>,
) -> ChainparserResult<AccountSharedData> {
    let lamports = u16::MAX as u64;
    let data = encoded_idl_data;
    let executable = false;
    let rent_epoch = Epoch::default();
    let account = Account {
        lamports,
        data,
        owner: program_pubkey,
        executable,
        rent_epoch,
    };
    Ok(account.into())
}
