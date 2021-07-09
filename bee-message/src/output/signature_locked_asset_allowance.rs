// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{convert::Infallible, fmt};

const ASSET_ID_LENGTH: usize = 32;

/// Error encountered packing a `SignatureLockedAssetAllowanceOutput`.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureLockedAssetAllowancePackError {
    InvalidPrefixLength,
}

impl From<PackPrefixError<Infallible, u32>> for SignatureLockedAssetAllowancePackError {
    fn from(_: PackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefixLength
    }
}

impl fmt::Display for SignatureLockedAssetAllowancePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for asset balance vector"),
        }
    }
}

/// Error encountered unpacking a `SignatureLockedAssetAllowanceOutput`.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureLockedAssetAllowanceUnpackError {
    InvalidAddressKind(u8),
    InvalidPrefixLength,
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    SignatureLockedAssetAllowanceUnpackError,
    ValidationError,
    SignatureLockedAssetAllowanceUnpackError::ValidationError
);

impl From<UnknownTagError<u8>> for SignatureLockedAssetAllowanceUnpackError {
    fn from(error: UnknownTagError<u8>) -> Self {
        Self::InvalidAddressKind(error.0)
    }
}

impl From<UnpackPrefixError<Infallible, u32>> for SignatureLockedAssetAllowanceUnpackError {
    fn from(_: UnpackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefixLength
    }
}

impl fmt::Display for SignatureLockedAssetAllowanceUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddressKind(kind) => write!(f, "Invalid address kind: {}", kind),
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for asset balance vector"),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// Tokenized asset balance information.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetBalance {
    /// The ID of the tokenized asset.
    id: [u8; ASSET_ID_LENGTH],
    /// The balance of the tokenized asset.
    balance: u64,
}

impl AssetBalance {
    /// Creates a new `AssetBalance`.
    pub fn new(id: [u8; 32], balance: u64) -> Self {
        Self { id, balance }
    }

    /// Returns the ID of an `AssetBalance`.
    pub fn id(&self) -> &[u8] {
        &self.id
    }

    /// Returns the balance of an `AssetBalance`.
    pub fn balance(&self) -> u64 {
        self.balance
    }
}

/// An output type which can be unlocked via a signature. It deposits onto one single address.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedAssetAllowanceOutput {
    address: Address,
    balances: Vec<AssetBalance>,
}

impl SignatureLockedAssetAllowanceOutput {
    /// The output kind of a `SignatureLockedAssetAllowanceOutput`.
    pub const KIND: u8 = 1;

    /// Creates a new `SignatureLockedAssetAllowanceOutput`.
    pub fn new(address: Address, balances: Vec<AssetBalance>) -> Result<Self, ValidationError> {
        Ok(Self { address, balances })
    }

    /// Returns the address of a `SignatureLockedAssetAllowanceOutput`.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a `SignatureLockedAssetAllowanceOutput`.
    pub fn balance_iter(&self) -> impl Iterator<Item = &AssetBalance> {
        self.balances.iter()
    }
}

impl Packable for SignatureLockedAssetAllowanceOutput {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + 0u32.packed_len() + self.balances.len() * (ASSET_ID_LENGTH + 0u64.packed_len())
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.address.pack(packer).map_err(PackError::infallible)?;

        let prefixed_balances = VecPrefix::<AssetBalance, u32>::from(self.balances.clone());
        prefixed_balances
            .pack(packer)
            .map_err(PackError::coerce::<SignatureLockedAssetAllowancePackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack(unpacker)
            .map_err(UnpackError::coerce::<SignatureLockedAssetAllowanceUnpackError>)
            .map_err(UnpackError::coerce)?;

        let balances = VecPrefix::<AssetBalance, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<SignatureLockedAssetAllowanceUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        Ok(Self { address, balances })
    }
}
