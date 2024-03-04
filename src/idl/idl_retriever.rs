use log::trace;
use solana_idl::Idl;
use solana_sdk::pubkey::Pubkey;

use super::{decode_idl_account_data, try_idl_address, IdlProvider};
use crate::{errors::ChainparserResult, traits::AccountProvider};

pub fn try_find_idl_for_program<T: AccountProvider>(
    account_provider: &T,
    program_id: &Pubkey,
    idl_provider: &IdlProvider,
) -> ChainparserResult<Option<Idl>> {
    let idl_address = try_idl_address(idl_provider, program_id)?;
    match account_provider.get_account(&idl_address) {
        Some((account, _)) => {
            let (idl, json) = decode_idl_account_data(&account.data)?;
            if std::option_env!("TRACE_RETRIEVED_IDL").is_some() {
                trace!("{}", json);
            }
            Ok(Some(idl))
        }
        None => Ok(None),
    }
}
