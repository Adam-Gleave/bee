// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{Error, MessageId};

use bee_ord::is_unique_sorted;
use bee_packable::{error::{PackPrefixError, UnpackPrefixError}, Packable, VecPrefix};

use core::{convert::Infallible, ops::{Deref, RangeInclusive}};

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

/// A [`Message`]'s `Parents` are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `MESSAGE_PARENTS_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible, u32>)]
#[packable(unpack_error = UnpackPrefixError<Infallible, u32>)]
pub struct Parents {
    #[packable(wrapper = VecPrefix<MessageId, u32>)]
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
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        if !MESSAGE_PARENTS_RANGE.contains(&inner.len()) {
            return Err(Error::InvalidParentsCount(inner.len()));
        }

        if !is_unique_sorted(inner.iter().map(AsRef::as_ref)) {
            return Err(Error::ParentsNotUniqueSorted);
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
