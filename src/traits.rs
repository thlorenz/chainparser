use solana_sdk::{account::Account, pubkey::Pubkey};

pub trait AccountProvider {
    fn get_account(&self, pubkey: &Pubkey) -> Option<(Account, u64)>;
}
