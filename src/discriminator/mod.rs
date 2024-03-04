pub mod match_discriminator;

use solana_sdk::hash::hash;

pub type DiscriminatorBytes = [u8; 8];

/// Derives the account discriminator form the account name using the same algorithm that anchor
/// uses.
pub fn account_discriminator(name: &str) -> DiscriminatorBytes {
    let mut discriminator = [0u8; 8];
    let hashed = hash(format!("account:{name}").as_bytes()).to_bytes();
    discriminator.copy_from_slice(&hashed[..8]);
    discriminator
}

pub fn discriminator_from_data(data: &[u8]) -> DiscriminatorBytes {
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&data[..8]);
    discriminator
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn account_discriminator_test() {
        let name = "VaultInfo";
        let discriminator = account_discriminator(name);
        assert_eq!(discriminator, [133, 250, 161, 78, 246, 27, 55, 187]);
    }
}
