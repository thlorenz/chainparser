use std::{collections::HashMap, ops::Deref};

use arrayref::array_ref;
use solana_idl::{IdlType, IdlTypeDefinition, IdlTypeDefinitionTy};

use crate::idl;

// -----------------
// Matcher
// -----------------
#[derive(Debug)]
pub enum Matcher {
    COption(usize, usize),
    Bool(usize),
}

impl TryFrom<(&IdlType, &HashMap<String, &IdlTypeDefinitionTy>, usize)>
    for Matcher
{
    type Error = ();

    fn try_from(
        (ty, type_map, offset): (
            &IdlType,
            &HashMap<String, &IdlTypeDefinitionTy>,
            usize,
        ),
    ) -> Result<Self, Self::Error> {
        match ty {
            IdlType::COption(inner) => {
                let inner_size =
                    idl::idl_type_bytes(inner, Some(type_map)).unwrap_or(0);
                Ok(Matcher::COption(offset, inner_size))
            }
            IdlType::Bool => Ok(Matcher::Bool(offset)),
            _ => Err(()),
        }
    }
}

impl Matcher {
    fn matches(&self, buf: &[u8]) -> bool {
        use Matcher::*;
        match self {
            COption(offset, _) => {
                let src = array_ref![buf, *offset, 4];
                matches!(src, [1, 0, 0, 0]) || matches!(src, [0, 0, 0, 0])
            }
            Bool(offset) => {
                let src = array_ref![buf, *offset, 1];
                matches!(src, [0] | [1])
            }
        }
    }
}

// -----------------
// MatchDiscriminators
// -----------------
#[derive(Debug)]
pub struct MatchDiscriminators(Vec<MatchDiscriminator>);
impl From<(&[IdlTypeDefinition], &HashMap<String, &IdlTypeDefinitionTy>)>
    for MatchDiscriminators
{
    fn from(
        (accounts, type_map): (
            &[IdlTypeDefinition],
            &HashMap<String, &IdlTypeDefinitionTy>,
        ),
    ) -> Self {
        let mut discs = accounts
            .iter()
            .flat_map(|acc| MatchDiscriminator::new(acc.clone(), type_map))
            .collect::<Vec<_>>();
        discs.sort_by_key(|f| f.min_total_size);
        Self(discs)
    }
}

impl Deref for MatchDiscriminators {
    type Target = Vec<MatchDiscriminator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MatchDiscriminators {
    pub fn find_match(&self, buf: &[u8]) -> Option<IdlTypeDefinition> {
        self.find_matching_disc(buf)
            .map(|disc| disc.account.clone())
    }

    pub fn find_match_name(&self, buf: &[u8]) -> Option<&str> {
        self.find_matching_disc(buf).map(|disc| disc.account_name())
    }

    fn find_matching_disc(&self, buf: &[u8]) -> Option<&MatchDiscriminator> {
        let mut candidates = Vec::new();
        for disc in self.iter() {
            if disc.matches_account(buf) {
                // if sizes match exactly as well then this is the best match
                if disc.min_total_size == buf.len() {
                    return Some(disc);
                } else {
                    candidates.push(disc);
                }
            }
        }
        // Did not find exact size match, thus we pick the discriminator
        // that had to match most fields
        let mut best_candidate = None::<&MatchDiscriminator>;
        for candidate in candidates {
            if let Some(disc) = best_candidate {
                if candidate.matchers.len() > disc.matchers.len() {
                    best_candidate = Some(candidate);
                }
            } else {
                best_candidate = Some(candidate);
            }
        }
        best_candidate
    }
}

// -----------------
// MatchDiscriminator
// -----------------
#[derive(Debug)]
pub struct MatchDiscriminator {
    pub account: IdlTypeDefinition,
    min_total_size: usize,
    matchers: Vec<Matcher>,
}

impl MatchDiscriminator {
    pub fn new(
        account: IdlTypeDefinition,
        type_map: &HashMap<String, &IdlTypeDefinitionTy>,
    ) -> Option<Self> {
        let account_sizes = base_account_sizes(&account, type_map);
        match account_sizes {
            Some((field_sizes, field_offsets)) => {
                let min_total_size = field_sizes.iter().sum();
                let matchers =
                    account_matchers(&account, type_map, &field_offsets);
                // TODO(thlorenz): should require at least have one multi byte matcher
                if matchers.is_empty() {
                    None
                } else {
                    Some(Self {
                        account,
                        min_total_size,
                        matchers,
                    })
                }
            }
            _ => None,
        }
    }

    pub fn account_name(&self) -> &str {
        &self.account.name
    }

    fn matches_account(&self, buf: &[u8]) -> bool {
        if buf.len() < self.min_total_size {
            return false;
        }
        self.matchers.iter().all(|matcher| matcher.matches(buf))
    }
}

fn account_matchers(
    account: &IdlTypeDefinition,
    type_map: &HashMap<String, &IdlTypeDefinitionTy>,
    offsets: &[usize],
) -> Vec<Matcher> {
    match &account.ty {
        IdlTypeDefinitionTy::Struct { fields } => {
            let mut matchers = Vec::new();
            for (field, offset) in fields.iter().zip(offsets) {
                if let Ok(matcher) =
                    Matcher::try_from((&field.ty, type_map, *offset))
                {
                    matchers.push(matcher)
                }
            }
            matchers
        }
        _ => Vec::new(),
    }
}

fn base_account_sizes(
    account: &IdlTypeDefinition,
    type_map: &HashMap<String, &IdlTypeDefinitionTy>,
) -> Option<(Vec<usize>, Vec<usize>)> {
    let mut offsets = Vec::new();
    let mut sizes = Vec::new();

    let mut offset = 0;

    match &account.ty {
        IdlTypeDefinitionTy::Struct { fields } => {
            for field in fields {
                if let Some(size) =
                    idl::idl_type_bytes(&field.ty, Some(type_map))
                {
                    offsets.push(offset);
                    sizes.push(size);
                    offset += size;
                }
            }
            Some((sizes, offsets))
        }
        _ => None, // accounts should always be structs
    }
}
