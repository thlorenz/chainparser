use std::str::FromStr;

use serde::{de::Error, Deserialize, Deserializer};
use solana_sdk::pubkey::Pubkey;

/// Function to provide to [serde] in order to deserialize a [Pubkey] from a base58 string.
/// Use as follows: `#[serde(deserialize_with = "pubkey_from_base58")]`
pub fn pubkey_from_base58<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(s).map_err(D::Error::custom)
}

/// Function to provide to [serde] in order to deserialize a [Vec<Pubkey>] from a a vec of base58
/// strings.
/// Use as follows: `#[serde(deserialize_with = "vec_pubkey_from_base58")]`
pub fn vec_pubkey_from_base58<'de, D>(
    deserializer: D,
) -> Result<Vec<Pubkey>, D::Error>
where
    D: Deserializer<'de>,
{
    let xs: Vec<&str> = Deserialize::deserialize(deserializer)?;
    xs.into_iter()
        .map(|s| Pubkey::from_str(s).map_err(D::Error::custom))
        .collect()
}

/// Function to provide to [serde] in order to deserialize a [Option<Pubkey>] from a base58 string
/// option.
/// Use as follows: `#[serde(deserialize_with = "opt_pubkey_from_base58")]`
pub fn opt_pubkey_from_base58<'de, D>(
    deserializer: D,
) -> Result<Option<Pubkey>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<&str> = Deserialize::deserialize(deserializer)?;
    opt.map(|s| Pubkey::from_str(s).map_err(D::Error::custom))
        .transpose()
}

/// Function to provide to [serde] in order to deserialize a [u64] from a string.
/// Use as follows: `#[serde(deserialize_with = "u64_from_string")]`
pub fn u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    u64::from_str(s).map_err(D::Error::custom)
}

/// Function to provide to [serde] in order to deserialize a [i64] from a string.
/// Use as follows: `#[serde(deserialize_with = "i64_from_string")]`
pub fn i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    i64::from_str(s).map_err(D::Error::custom)
}

/// Function to provide to [serde] in order to deserialize a [u128] from a string.
/// Use as follows: `#[serde(deserialize_with = "u128_from_string")]`
pub fn u128_from_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    u128::from_str(s).map_err(D::Error::custom)
}

/// Function to provide to [serde] in order to deserialize a [i128] from a string.
/// Use as follows: `#[serde(deserialize_with = "i128_from_string")]`
pub fn i128_from_string<'de, D>(deserializer: D) -> Result<i128, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    i128::from_str(s).map_err(D::Error::custom)
}
