// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, error::ValidationError};

use bee_packable::{Packable, UnknownTagError};

use core::ops::RangeInclusive;

/// Valid amounts for a signature locked single output.
pub const SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

/// An output type which can be unlocked via a signature. It deposits onto one single address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = UnknownTagError<u8>)]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedSingleOutput {
    /// The output kind of a `SignatureLockedSingleOutput`.
    pub const KIND: u8 = 0;

    /// Creates a new `SignatureLockedSingleOutput`.
    pub fn new(address: Address, amount: u64) -> Result<Self, ValidationError> {
        if !SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT.contains(&amount) {
            return Err(ValidationError::InvalidAmount(amount));
        }

        Ok(Self { address, amount })
    }

    /// Returns the address of a `SignatureLockedSingleOutput`.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a `SignatureLockedSingleOutput`.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}
