// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

use crate::{unlock::UnlockBlocks, Error};

pub use essence::{TransactionEssence, TransactionEssenceBuilder, TransactionEssencePackError, TransactionEssenceUnpackError};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_packable::{PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker, error::{PackPrefixError, UnpackPrefixError}};
use crypto::hashes::{blake2b::Blake2b256, Digest};

use core::convert::Infallible;

#[derive(Debug)]
pub enum TransactionPackError {
    TransactionEssence,
    UnlockBlocks,
}

impl From<TransactionEssencePackError> for TransactionPackError {
    fn from(_: TransactionEssencePackError) -> Self {
        Self::TransactionEssence
    }
}

impl From<PackPrefixError<Infallible, u16>> for TransactionPackError {
    fn from(_: PackPrefixError<Infallible, u16>) -> Self {
        Self::UnlockBlocks
    }
}

#[derive(Debug)]
pub enum TransactionUnpackError {
    TransactionEssence,
    UnlockBlocks,
}

impl From<TransactionEssenceUnpackError> for TransactionUnpackError {
    fn from(_: TransactionEssenceUnpackError) -> Self {
        Self::TransactionEssence 
    }
}

impl From<UnpackPrefixError<UnknownTagError<u8>, u16>> for TransactionUnpackError {
    fn from(_: UnpackPrefixError<UnknownTagError<u8>, u16>) -> Self {
        Self::UnlockBlocks
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

        Ok(Self {
            essence,
            unlock_blocks,
        })
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
    pub fn finish(self) -> Result<TransactionPayload, Error> {
        let essence = self.essence.ok_or(Error::MissingField("essence"))?;
        let unlock_blocks = self.unlock_blocks.ok_or(Error::MissingField("unlock_blocks"))?;

        if essence.inputs().len() != unlock_blocks.len() {
            return Err(Error::InputUnlockBlockCountMismatch(
                essence.inputs().len(),
                unlock_blocks.len(),
            ));
        }

        Ok(TransactionPayload { essence, unlock_blocks })
    }
}
