// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;

pub use reference::ReferenceUnlock;

use crate::{constants::UNLOCK_BLOCK_COUNT_RANGE, error::ValidationError, signature::SignatureUnlock};

use bee_packable::{error::{PackPrefixError, UnpackPrefixError}, Packable, UnknownTagError, VecPrefix};

use core::{convert::Infallible, ops::Deref};
use std::collections::HashSet;

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8)]
pub enum UnlockBlock {
    /// A signature unlock block.
    #[packable(tag = 0)]
    Signature(SignatureUnlock),
    /// A reference unlock block.
    #[packable(tag = 1)]
    Reference(ReferenceUnlock),
}

impl UnlockBlock {
    /// Returns the unlock kind of an `UnlockBlock`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Signature(_) => SignatureUnlock::KIND,
            Self::Reference(_) => ReferenceUnlock::KIND,
        }
    }
}

impl From<SignatureUnlock> for UnlockBlock {
    fn from(signature: SignatureUnlock) -> Self {
        Self::Signature(signature)
    }
}

impl From<ReferenceUnlock> for UnlockBlock {
    fn from(reference: ReferenceUnlock) -> Self {
        Self::Reference(reference)
    }
}

/// A collection of unlock blocks.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible, u16>)]
#[packable(unpack_error = UnpackPrefixError<UnknownTagError<u8>, u16>)]
pub struct UnlockBlocks {
    #[packable(wrapper = VecPrefix<UnlockBlock, u16>)]
    inner: Vec<UnlockBlock>,
}

impl UnlockBlocks {
    /// Creates a new `UnlockBlocks`.
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, ValidationError> {
        if !UNLOCK_BLOCK_COUNT_RANGE.contains(&unlock_blocks.len()) {
            return Err(ValidationError::InvalidUnlockBlockCount(unlock_blocks.len()));
        }

        let mut seen_signatures = HashSet::new();

        for (index, unlock_block) in unlock_blocks.iter().enumerate() {
            match unlock_block {
                UnlockBlock::Reference(r) => {
                    if index == 0
                        || r.index() >= index as u16
                        || matches!(unlock_blocks[r.index() as usize], UnlockBlock::Reference(_))
                    {
                        return Err(ValidationError::InvalidUnlockBlockReference(index));
                    }
                }
                UnlockBlock::Signature(s) => {
                    if !seen_signatures.insert(s) {
                        return Err(ValidationError::DuplicateSignature(index));
                    }
                }
            }
        }

        Ok(Self { inner: unlock_blocks.into() })
    }

    /// Gets an `UnlockBlock` from an `UnlockBlocks`.
    /// Returns the referenced unlock block if the requested unlock block was a reference.
    pub fn get(&self, index: usize) -> Option<&UnlockBlock> {
        match self.inner.get(index) {
            Some(UnlockBlock::Reference(reference)) => self.inner.get(reference.index() as usize),
            Some(unlock_block) => Some(unlock_block),
            None => None,
        }
    }
}

impl Deref for UnlockBlocks {
    type Target = [UnlockBlock];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}
