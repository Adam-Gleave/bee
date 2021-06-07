// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{Error, MESSAGE_LENGTH_MAX};

pub use padded::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH};

use bee_packable::{Packable, VecPrefix};

use core::ops::RangeInclusive;

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<usize> = 1..=INDEXATION_PADDED_INDEX_LENGTH;

/// A payload which holds an index and associated data.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationPayload {
    version: u8,
    index: VecPrefix<u8, u16>,
    data: VecPrefix<u8, u16>,
}

impl IndexationPayload {
    /// The payload kind of an `IndexationPayload`.
    pub const KIND: u32 = 8;

    /// Creates a new `IndexationPayload`.
    pub fn new(version: u8, index: Vec<u8>, data: Vec<u8>) -> Result<Self, Error> {
        if !INDEXATION_INDEX_LENGTH_RANGE.contains(&index.len()) {
            return Err(Error::InvalidIndexationIndexLength(index.len()));
        }

        if data.len() > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidIndexationDataLength(data.len()));
        }

        Ok(Self {
            version,
            index: index.into(),
            data: data.into(),
        })
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
