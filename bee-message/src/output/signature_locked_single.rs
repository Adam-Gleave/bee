// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, error::ValidationError};

use bee_packable::{Packable, Packer, PackError, Unpacker, UnpackError, UnknownTagError};

use core::{fmt, convert::Infallible, ops::RangeInclusive};

/// Valid amounts for a signature locked single output.
pub const SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

#[derive(Debug)]
pub enum SignatureLockedSingleOutputUnpackError {
    InvalidAddressKind(u8),
    ValidationError(ValidationError),
}

impl From<UnknownTagError<u8>> for SignatureLockedSingleOutputUnpackError {
    fn from(error: UnknownTagError<u8>) -> Self {
        Self::InvalidAddressKind(error.0) 
    }
}

impl From<ValidationError> for SignatureLockedSingleOutputUnpackError {
    fn from(error: ValidationError) -> Self {
        Self::ValidationError(error)
    }
}

impl fmt::Display for SignatureLockedSingleOutputUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddressKind(kind) => write!(f, "Invalid address kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// An output type which can be unlocked via a signature. It deposits onto one single address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedSingleOutput {
    /// The output kind of a `SignatureLockedSingleOutput`.
    pub const KIND: u8 = 0;

    /// Creates a new `SignatureLockedSingleOutput`.
    pub fn new(address: Address, amount: u64) -> Result<Self, ValidationError> {
        validate_amount(amount)?;

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

impl Packable for SignatureLockedSingleOutput {
    type PackError = Infallible;
    type UnpackError = SignatureLockedSingleOutputUnpackError;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + self.amount.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.address.pack(packer).map_err(PackError::infallible)?;
        self.amount.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack(unpacker).map_err(UnpackError::coerce)?;
        let amount = u64::unpack(unpacker).map_err(UnpackError::infallible)?;

        validate_amount(amount).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { address, amount })
    }
}

fn validate_amount(amount: u64) -> Result<(), ValidationError> {
    if !SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT.contains(&amount) {
        Err(ValidationError::InvalidAmount(amount))
    } else {
        Ok(())
    }
}
