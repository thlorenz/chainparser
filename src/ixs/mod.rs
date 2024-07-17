use solana_sdk::pubkey::Pubkey;

mod discriminator;
mod instruction_mapper;

pub trait ParseableInstruction {
    fn program_id(&self) -> &Pubkey;
    fn accounts(&self) -> Vec<Pubkey>;
    fn data(&self) -> &[u8];
}

pub use instruction_mapper::{
    map_instruction, InstructionMapResult, InstructionMapper, BUILTIN_PROGRAMS,
};
