// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{MessagePackError, MessageUnpackError, ValidationError, MessageId};

use bee_ord::is_unique_sorted;
use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    Packable, Packer, PackError, Unpacker, UnpackError, VecPrefix,
};

use core::{
    fmt,
    convert::Infallible,
    ops::{Deref, RangeInclusive},
};

use alloc::vec::Vec;

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

#[derive(Debug)]
pub enum ParentsPackError {
    InvalidPrefixLength,
}

impl From<PackPrefixError<Infallible, u32>> for ParentsPackError {
    fn from(_: PackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefixLength
    }
}

impl fmt::Display for ParentsPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for message parents"),
        }
    }
}

#[derive(Debug)]
pub enum ParentsUnpackError {
    InvalidPrefixLength,
    ValidationError(ValidationError),
}

impl From<ValidationError> for ParentsUnpackError {
    fn from(error: ValidationError) -> Self {
        Self::ValidationError(error)
    }
}

impl From<UnpackPrefixError<Infallible, u32>> for ParentsUnpackError {
    fn from(_: UnpackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefixLength
    }
}

impl fmt::Display for ParentsUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for message parents"),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A [`Message`]'s `Parents` are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `MESSAGE_PARENTS_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents {
    inner: Vec<MessageId>,
}

impl Deref for Parents {
    type Target = [MessageId];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// Creates new `Parents`.
    pub fn new(inner: Vec<MessageId>) -> Result<Self, ValidationError> {
        if !MESSAGE_PARENTS_RANGE.contains(&inner.len()) {
            return Err(ValidationError::InvalidParentsCount(inner.len()));
        }

        if !is_unique_sorted(inner.iter().map(AsRef::as_ref)) {
            return Err(ValidationError::ParentsNotUniqueSorted);
        }

        Ok(Self { inner })
    }

    /// Returns the number of parents.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an iterator over the parents.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &MessageId> + '_ {
        self.inner.iter()
    }
}

impl Packable for Parents {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        VecPrefix::<MessageId, u32>::from(self.inner.clone()).packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        let prefixed = VecPrefix::<MessageId, u32>::from(self.inner.clone());
        prefixed.pack(packer).map_err(PackError::coerce::<ParentsPackError>).map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let parents = VecPrefix::<MessageId, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<ParentsUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        Ok(Parents::new(parents).map_err(|e| UnpackError::Packable(e.into()))?)
    }
}
