// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, error::{MessageUnpackError, ValidationError}};

use bee_packable::{PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker};

use core::{fmt, convert::Infallible, ops::RangeInclusive};

/// Amount of tokens below which an output is considered a dust output.
pub const DUST_THRESHOLD: u64 = 1_000_000;
/// Valid amounts for a signature locked dust allowance output.
pub const SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT: RangeInclusive<u64> = DUST_THRESHOLD..=IOTA_SUPPLY;

#[derive(Debug)]
pub enum SignatureLockedDustAllowanceUnpackError {
    InvalidAddressKind(u8),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    SignatureLockedDustAllowanceUnpackError, 
    ValidationError, 
    SignatureLockedDustAllowanceUnpackError::ValidationError
);

impl From<UnknownTagError<u8>> for SignatureLockedDustAllowanceUnpackError {
    fn from(error: UnknownTagError<u8>) -> Self {
        match error {
            UnknownTagError(tag) => Self::InvalidAddressKind(tag)
        }
    }
}

impl fmt::Display for SignatureLockedDustAllowanceUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddressKind(kind) => write!(f, "Invalid address kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A `SignatureLockedDustAllowanceOutput` functions like a `SignatureLockedSingleOutput` but as a special property it
/// is used to increase the allowance/amount of dust outputs on a given address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedDustAllowanceOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedDustAllowanceOutput {
    /// The output kind of a `SignatureLockedDustAllowanceOutput`.
    pub const KIND: u8 = 1;

    /// Creates a new `SignatureLockedDustAllowanceOutput`.
    pub fn new(address: Address, amount: u64) -> Result<Self, ValidationError> {
        validate_amount(amount)?;

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

impl Packable for SignatureLockedDustAllowanceOutput {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + self.amount.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.address.pack(packer).map_err(PackError::infallible)?;
        self.amount.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack(unpacker)
            .map_err(UnpackError::coerce::<SignatureLockedDustAllowanceUnpackError>)
            .map_err(UnpackError::coerce)?;
        
        let amount = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_amount(amount).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { address, amount })
    }
}

fn validate_amount(amount: u64) -> Result<(), ValidationError> {
    if !SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT.contains(&amount) {
        return Err(ValidationError::InvalidDustAllowanceAmount(amount))
    } else {
        Ok(())
    }
}
