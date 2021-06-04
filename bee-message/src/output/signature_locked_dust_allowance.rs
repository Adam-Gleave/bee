// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, Error};

use bee_packable::{Packable, UnknownTagError};

use core::ops::RangeInclusive;

/// Amount of tokens below which an output is considered a dust output.
pub const DUST_THRESHOLD: u64 = 1_000_000;
/// Valid amounts for a signature locked dust allowance output.
pub const SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT: RangeInclusive<u64> = DUST_THRESHOLD..=IOTA_SUPPLY;

/// A `SignatureLockedDustAllowanceOutput` functions like a `SignatureLockedSingleOutput` but as a special property it
/// is used to increase the allowance/amount of dust outputs on a given address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(error = UnknownTagError<u8>)]
pub struct SignatureLockedDustAllowanceOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedDustAllowanceOutput {
    /// The output kind of a `SignatureLockedDustAllowanceOutput`.
    pub const KIND: u8 = 1;

    /// Creates a new `SignatureLockedDustAllowanceOutput`.
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        if !SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT.contains(&amount) {
            return Err(Error::InvalidDustAllowanceAmount(amount));
        }

        Ok(Self { address, amount })
    }

    /// Returns the address of a `SignatureLockedDustAllowanceOutput`.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a `SignatureLockedDustAllowanceOutput`.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}
