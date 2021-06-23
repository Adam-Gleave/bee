// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

use crate::{error::ValidationError, unlock::{UnlockBlockUnpackError, UnlockBlocks}};

pub use essence::{
    TransactionEssence, TransactionEssenceBuilder, TransactionEssencePackError, TransactionEssenceUnpackError,
};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker,
};
use crypto::hashes::{blake2b::Blake2b256, Digest};

use alloc::boxed::Box;
use core::{convert::Infallible, fmt};

#[derive(Debug)]
pub enum TransactionPackError {
    InvalidUnlockBlocksPrefix,
    TransactionEssence(Box<TransactionEssencePackError>),
}

impl fmt::Display for TransactionPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnlockBlocksPrefix => write!(f, "Invalid unlock block vector prefix"),
            Self::TransactionEssence(e) => write!(f, "{}", e),
        }
    }
}

impl From<TransactionEssencePackError> for TransactionPackError {
    fn from(error: TransactionEssencePackError) -> Self {
        Self::TransactionEssence(Box::new(error))
    }
}

impl From<PackPrefixError<Infallible, u16>> for TransactionPackError {
    fn from(error: PackPrefixError<Infallible, u16>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {}
            PackPrefixError::Prefix(_) => Self::InvalidUnlockBlocksPrefix,
        }
    }
}

#[derive(Debug)]
pub enum TransactionUnpackError {
    InvalidUnlockBlocksPrefix,
    TransactionEssence(Box<TransactionEssenceUnpackError>),
    UnlockBlockUnpack(UnlockBlockUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(TransactionUnpackError, ValidationError, TransactionUnpackError::ValidationError);

impl fmt::Display for TransactionUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnlockBlocksPrefix => write!(f, "Invalid unlock block vector prefix"),
            Self::TransactionEssence(e) => write!(f, "Error unpacking transaction essence: {}", e),
            Self::UnlockBlockUnpack(e) => write!(f, "Error unpacking unlock blocks: {}", e),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

impl From<TransactionEssenceUnpackError> for TransactionUnpackError {
    fn from(error: TransactionEssenceUnpackError) -> Self {
        match error {
            TransactionEssenceUnpackError::ValidationError(error) => Self::ValidationError(error),
            error => Self::TransactionEssence(Box::new(error)),
        }
    }
}

impl From<UnpackPrefixError<UnlockBlockUnpackError, u16>> for TransactionUnpackError {
    fn from(error: UnpackPrefixError<UnlockBlockUnpackError, u16>) -> Self {
        match error {
            UnpackPrefixError::Packable(error) => match error {
                UnlockBlockUnpackError::ValidationError(error) => Self::ValidationError(error),
                error => Self::UnlockBlockUnpack(error),
            }
            UnpackPrefixError::Prefix(_) => Self::InvalidUnlockBlocksPrefix,
        }
    }
}

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionPayload {
    essence: TransactionEssence,
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    /// The payload kind of a `TransactionPayload`.
    pub const KIND: u32 = 1;

    /// Return a new `TransactionPayloadBuilder` to build a `TransactionPayload`.
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    /// Computes the identifier of a `TransactionPayload`.
    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();
        hasher.update(Self::KIND.to_le_bytes());

        let bytes = self.pack_to_vec().unwrap();

        hasher.update(bytes);

        TransactionId::new(hasher.finalize().into())
    }

    /// Return the essence of a `TransactionPayload`.
    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    /// Return unlock blocks of a `TransactionPayload`.
    pub fn unlock_blocks(&self) -> &UnlockBlocks {
        &self.unlock_blocks
    }
}

impl Packable for TransactionPayload {
    type PackError = TransactionPackError;
    type UnpackError = TransactionUnpackError;

    fn packed_len(&self) -> usize {
        self.essence.packed_len() + self.unlock_blocks.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.essence.pack(packer).map_err(PackError::coerce)?;
        self.unlock_blocks.pack(packer).map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let essence = TransactionEssence::unpack(unpacker).map_err(UnpackError::coerce)?;
        let unlock_blocks = UnlockBlocks::unpack(unpacker).map_err(UnpackError::coerce)?;

        validate_unlock_block_count(&essence, &unlock_blocks).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { essence, unlock_blocks })
    }
}

/// A builder to build a `TransactionPayload`.
#[derive(Debug, Default)]
pub struct TransactionPayloadBuilder {
    essence: Option<TransactionEssence>,
    unlock_blocks: Option<UnlockBlocks>,
}

impl TransactionPayloadBuilder {
    /// Creates a new `TransactionPayloadBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an essence to a `TransactionPayloadBuilder`.
    pub fn with_essence(mut self, essence: TransactionEssence) -> Self {
        self.essence.replace(essence);
        self
    }

    /// Adds unlock blocks to a `TransactionPayloadBuilder`.
    pub fn with_unlock_blocks(mut self, unlock_blocks: UnlockBlocks) -> Self {
        self.unlock_blocks.replace(unlock_blocks);
        self
    }

    /// Finishes a `TransactionPayloadBuilder` into a `TransactionPayload`.
    pub fn finish(self) -> Result<TransactionPayload, ValidationError> {
        let essence = self.essence.ok_or(ValidationError::MissingField("essence"))?;
        let unlock_blocks = self
            .unlock_blocks
            .ok_or(ValidationError::MissingField("unlock_blocks"))?;

        validate_unlock_block_count(&essence, &unlock_blocks)?;

        Ok(TransactionPayload { essence, unlock_blocks })
    }
}

fn validate_unlock_block_count(
    essence: &TransactionEssence, 
    unlock_blocks: &UnlockBlocks,
) -> Result<(), ValidationError> {
    if essence.inputs().len() != unlock_blocks.len() {
        Err(ValidationError::InputUnlockBlockCountMismatch(
            essence.inputs().len(),
            unlock_blocks.len(),
        ))
    } else {
        Ok(())
    }
}
