use solana_sdk::pubkey::Pubkey;

mod accounts_mapper;
mod discriminator;

pub trait ParseableInstruction {
    fn program_id(&self) -> &Pubkey;
    fn accounts(&self) -> Vec<Pubkey>;
    fn data(&self) -> &[u8];
}

pub use accounts_mapper::map_instruction_account_labels;
