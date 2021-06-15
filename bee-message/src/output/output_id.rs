// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::INPUT_OUTPUT_INDEX_RANGE,
    error::ValidationError,
    payload::transaction::{TransactionId, TRANSACTION_ID_LENGTH},
};

use bee_packable::Packable;

use core::{
    convert::{From, TryFrom, TryInto},
    str::FromStr,
};

/// The length of an `OutputId`.
pub const OUTPUT_ID_LENGTH: usize = TRANSACTION_ID_LENGTH + std::mem::size_of::<u16>();

/// The identifier of an `Output`.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
pub struct OutputId {
    transaction_id: TransactionId,
    index: u16,
}

impl OutputId {
    /// Creates a new `OutputId`.
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, ValidationError> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(ValidationError::InvalidOutputIndex(index));
        }

        Ok(Self { transaction_id, index })
    }

    /// Returns the `TransactionId` of an `OutputId`.
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the index of an `OutputId`.
    pub fn index(&self) -> u16 {
        self.index
    }

    /// Splits an `OutputId` into its `TransactionId` and index.
    pub fn split(self) -> (TransactionId, u16) {
        (self.transaction_id, self.index)
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(OutputId);

impl TryFrom<[u8; OUTPUT_ID_LENGTH]> for OutputId {
    type Error = ValidationError;

    fn try_from(bytes: [u8; OUTPUT_ID_LENGTH]) -> Result<Self, Self::Error> {
        let (transaction_id, index) = bytes.split_at(TRANSACTION_ID_LENGTH);

        Self::new(
            // Unwrap is fine because size is already known and valid.
            From::<[u8; TRANSACTION_ID_LENGTH]>::from(transaction_id.try_into().unwrap()),
            // Unwrap is fine because size is already known and valid.
            u16::from_le_bytes(index.try_into().unwrap()),
        )
    }
}

impl FromStr for OutputId {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; OUTPUT_ID_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(OUTPUT_ID_LENGTH * 2, s.len()))?;

        bytes.try_into()
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.transaction_id, hex::encode(self.index.to_le_bytes()))
    }
}

impl core::fmt::Debug for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OutputId({})", self)
    }
}
