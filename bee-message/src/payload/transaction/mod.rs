// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the transaction payload.

mod essence;
mod transaction_id;

use crate::{unlock::UnlockBlocks, Error};

pub use essence::{TransactionEssence, TransactionEssenceBuilder, TransactionUnpackError};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};

use bee_packable::{Packable, VecPacker};
use crypto::hashes::{blake2b::Blake2b256, Digest};

use core::ops::Deref;

/// A transaction to move funds.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(error = crate::Error)]
pub struct TransactionPayload {
    essence: TransactionEssence,
    unlock_blocks: UnlockBlocks,
}

impl TransactionPayload {
    /// The payload kind of a `TransactionPayload`.
    pub const KIND: u32 = 0;

    /// Return a new `TransactionPayloadBuilder` to build a `TransactionPayload`.
    pub fn builder() -> TransactionPayloadBuilder {
        TransactionPayloadBuilder::default()
    }

    /// Computes the identifier of a `TransactionPayload`.
    pub fn id(&self) -> TransactionId {
        let mut hasher = Blake2b256::new();
        hasher.update(Self::KIND.to_le_bytes());

        let mut bytes = VecPacker::new();
        self.pack(&mut bytes).unwrap();
        let vec_bytes = bytes.deref().clone();

        hasher.update(vec_bytes);

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
