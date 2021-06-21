// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;

pub use reference::ReferenceUnlock;

use crate::{
    constants::UNLOCK_BLOCK_COUNT_RANGE, 
    error::ValidationError,
    signature::SignatureUnlock,
    unlock::reference::ReferenceUnlockUnpackError,
};

use bee_packable::{Packable, Packer, PackError, Unpacker, UnpackError, UnknownTagError, VecPrefix, error::{PackPrefixError, UnpackPrefixError}};

use core::{fmt, convert::Infallible, ops::Deref};
use std::collections::HashSet;

#[derive(Debug)]
pub enum UnlockBlockUnpackError {
    InvalidUnlockBlockKind(u8),
    InvalidSignatureUnlockKind(u8),
    ReferenceUnlock(ReferenceUnlockUnpackError),
}

impl From<ReferenceUnlockUnpackError> for UnlockBlockUnpackError {
    fn from(error: ReferenceUnlockUnpackError) -> Self {
        Self::ReferenceUnlock(error)
    }
}

impl From<UnknownTagError<u8>> for UnlockBlockUnpackError {
    fn from(error: UnknownTagError<u8>) -> Self {
        Self::InvalidSignatureUnlockKind(error.0)
    }
}

impl fmt::Display for UnlockBlockUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnlockBlockKind(kind) => write!(f, "Invalid unlock block kind: {}", kind),
            Self::InvalidSignatureUnlockKind(kind) => write!(f, "Invalid signature unlock kind: {}", kind),
            Self::ReferenceUnlock(e) => write!(f, "Error unpacking ReferenceUnlock: {}", e),
        }
    }
}

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum UnlockBlock {
    /// A signature unlock block.
    Signature(SignatureUnlock),
    /// A reference unlock block.
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

impl Packable for UnlockBlock {
    type PackError = Infallible;
    type UnpackError = UnlockBlockUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + match self {
            Self::Signature(signature) => signature.packed_len(),
            Self::Reference(reference) => reference.packed_len(),
        }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.kind().pack(packer).map_err(PackError::infallible)?;

        match self {
            Self::Signature(signature) => signature.pack(packer).map_err(PackError::infallible)?,
            Self::Reference(reference) => reference.pack(packer).map_err(PackError::infallible)?,
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let kind = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let variant = match kind {
            SignatureUnlock::KIND => Self::Signature(SignatureUnlock::unpack(unpacker).map_err(UnpackError::coerce)?),
            ReferenceUnlock::KIND => Self::Reference(ReferenceUnlock::unpack(unpacker).map_err(UnpackError::coerce)?),
            tag => Err(UnpackError::Packable(UnlockBlockUnpackError::InvalidUnlockBlockKind(tag)))?,
        };

        Ok(variant)
    }
}

/// A collection of unlock blocks.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible, u16>)]
#[packable(unpack_error = UnpackPrefixError<UnlockBlockUnpackError, u16>)]
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

        Ok(Self {
            inner: unlock_blocks.into(),
        })
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
