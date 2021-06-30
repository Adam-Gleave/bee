// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{MESSAGE_ID_LENGTH, MessageId, MessagePackError, MessageUnpackError, ValidationError};

use bee_ord::is_unique_sorted;
use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    Packable, Packer, PackError, Unpacker, UnpackError,
};

use bitvec::prelude::*;

use core::{
    fmt,
    convert::Infallible,
    ops::{Deref, RangeInclusive},
};

use alloc::vec;
use alloc::vec::Vec;

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

pub const MESSAGE_MIN_STRONG_PARENTS: usize = 1;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parent {
    Strong(MessageId),
    Weak(MessageId),
}

impl Parent {
    pub fn id(&self) -> &MessageId {
        match self {
            Self::Strong(id) => id,
            Self::Weak(id) => id,
        }
    }
}

// /// A [`Message`]'s `Parents` are the [`MessageId`]s of the messages it directly approves.
// ///
// /// Parents must be:
// /// * in the `MESSAGE_PARENTS_RANGE` range;
// /// * lexicographically sorted;
// /// * unique;
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents { 
    inner: Vec<Parent>,
}

impl Deref for Parents {
    type Target = [Parent];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

impl Parents {
    pub fn new(inner: Vec<Parent>) -> Result<Self, ValidationError> {
        validate_parents_count(inner.len())?;
        validate_parents_unique_sorted(&inner)?;

        let strong_count = inner
            .iter()
            .fold(0usize, |acc, parent| {
                match parent {
                    Parent::Strong(_) => acc + 1,
                    _ => acc,
                }
            });

        validate_strong_parents_count(strong_count)?;
        
        Ok(Self { inner })
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn strong_iter(&self) -> impl Iterator<Item = &MessageId> + '_ {
        self.inner
            .iter()
            .filter(|parent| { 
                match parent {
                    Parent::Strong(_) => true,
                    _ => false,
                } 
            })
            .map(Parent::id)
    }

    pub fn weak_iter(&self) -> impl Iterator<Item = &MessageId> + '_ {
        self.inner
            .iter()
            .filter(|parent| {
                match parent {
                    Parent::Weak(_) => true,
                    _ => false,
                }
            })
            .map(Parent::id)
    }
}

impl Packable for Parents {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + 0u8.packed_len()
            + self.inner.len() * MESSAGE_ID_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        (self.len() as usize).pack(packer).map_err(PackError::infallible)?;
        
        let mut bits = bitarr![Lsb0, u8; 0; 8];

        for (i, parent) in self.iter().enumerate() {
            let is_strong = match parent {
                Parent::Strong(_) => true,
                _ => false,
            };

            bits.set(i, is_strong);
        }

        let bits_repr = bits.load::<u8>();
        bits_repr.pack(packer).map_err(PackError::infallible)?;

        for id in self.iter().map(Parent::id) {
            id.pack(packer).map_err(PackError::infallible)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let count = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_parents_count(count as usize).map_err(|e| UnpackError::Packable(e.into()))?;

        let bits_repr = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];
        bits.store(bits_repr);
        validate_parents_count(bits.count_ones()).map_err(|e| UnpackError::Packable(e.into()))?;

        let mut parents = vec![];

        for i in 0..8 {
            if i < count {
                let id = MessageId::unpack(unpacker).map_err(UnpackError::infallible)?;

                if *bits.get(i as usize).unwrap() {
                    parents.push(Parent::Strong(id))
                } else {
                    parents.push(Parent::Weak(id))
                }
            } else {
                MessageId::unpack(unpacker).map_err(UnpackError::infallible)?;
            }
        }

        validate_parents_unique_sorted(&parents).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { inner: parents })
    }
}

fn validate_parents_count(count: usize) -> Result<(), ValidationError> {
    if !MESSAGE_PARENTS_RANGE.contains(&count) {
        Err(ValidationError::InvalidParentsCount(count))
    } else {
        Ok(())
    }
}

fn validate_strong_parents_count(count: usize) -> Result<(), ValidationError> {
    if count < MESSAGE_MIN_STRONG_PARENTS {
        Err(ValidationError::InvalidStrongParentsCount(count))
    } else {
        Ok(())
    }
}

fn validate_parents_unique_sorted(parents: &Vec<Parent>) -> Result<(), ValidationError> {
    if !is_unique_sorted(parents.iter().map(|parent| parent.id().as_ref())) {
        Err(ValidationError::ParentsNotUniqueSorted)
    } else {
        Ok(())
    }
}
