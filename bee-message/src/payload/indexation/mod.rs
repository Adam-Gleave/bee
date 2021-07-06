// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{MessagePackError, MessageUnpackError, ValidationError, MESSAGE_LENGTH_RANGE};

pub use padded::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{convert::Infallible, fmt, ops::RangeInclusive};

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<usize> = 1..=INDEXATION_PADDED_INDEX_LENGTH;

#[derive(Debug)]
pub enum IndexationPackError {
    InvalidPrefixLength,
}

impl From<PackPrefixError<Infallible, u32>> for IndexationPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefixLength,
        }
    }
}

impl fmt::Display for IndexationPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for index/data"),
        }
    }
}

#[derive(Debug)]
pub enum IndexationUnpackError {
    InvalidPrefixLength,
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    IndexationUnpackError,
    ValidationError,
    IndexationUnpackError::ValidationError
);

impl From<UnpackPrefixError<Infallible, u32>> for IndexationUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefixLength,
        }
    }
}

impl fmt::Display for IndexationUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for index/data"),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A payload which holds an index and associated data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationPayload {
    version: u8,
    index: Vec<u8>,
    data: Vec<u8>,
}

impl IndexationPayload {
    /// The payload kind of an `IndexationPayload`.
    pub const KIND: u32 = 8;

    /// Creates a new `IndexationPayload`.
    pub fn new(version: u8, index: Vec<u8>, data: Vec<u8>) -> Result<Self, ValidationError> {
        validate_index(&index)?;
        validate_data(&data)?;

        Ok(Self { version, index, data })
    }

    /// Returns the index of an `IndexationPayload`.
    pub fn index(&self) -> &[u8] {
        &self.index
    }

    /// Returns the padded index of an `IndexationPayload`.
    pub fn padded_index(&self) -> PaddedIndex {
        let mut padded_index = [0u8; INDEXATION_PADDED_INDEX_LENGTH];
        padded_index[..self.index.len()].copy_from_slice(&self.index);
        PaddedIndex::from(padded_index)
    }

    /// Returns the data of an `IndexationPayload`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

fn validate_index(index: &[u8]) -> Result<(), ValidationError> {
    if !INDEXATION_INDEX_LENGTH_RANGE.contains(&index.len()) {
        Err(ValidationError::InvalidIndexationIndexLength(index.len()))
    } else {
        Ok(())
    }
}

fn validate_data(data: &[u8]) -> Result<(), ValidationError> {
    if data.len() > *MESSAGE_LENGTH_RANGE.end() {
        Err(ValidationError::InvalidIndexationDataLength(data.len()))
    } else {
        Ok(())
    }
}

impl Packable for IndexationPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + VecPrefix::<u8, u32>::from(self.index.clone()).packed_len()
            + VecPrefix::<u8, u32>::from(self.data.clone()).packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;

        let prefixed_index: VecPrefix<u8, u32> = self.index.clone().into();
        prefixed_index
            .pack(packer)
            .map_err(PackError::coerce::<IndexationPackError>)
            .map_err(PackError::coerce)?;

        let prefixed_data: VecPrefix<u8, u32> = self.data.clone().into();
        prefixed_data
            .pack(packer)
            .map_err(PackError::coerce::<IndexationPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let index: Vec<u8> = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<IndexationUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        validate_index(&index).map_err(|e| UnpackError::Packable(e.into()))?;

        let data: Vec<u8> = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<IndexationUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        validate_data(&data).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { version, index, data })
    }
}
