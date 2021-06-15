// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::INPUT_OUTPUT_INDEX_RANGE, error::ValidationError};

use bee_packable::Packable;

use core::convert::TryFrom;

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) that refers to another unlock block.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReferenceUnlock(u16);

impl ReferenceUnlock {
    /// The unlock kind of a `ReferenceUnlock`.
    pub const KIND: u8 = 1;

    /// Creates a new `ReferenceUnlock`.
    pub fn new(index: u16) -> Result<Self, ValidationError> {
        if !INPUT_OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(ValidationError::InvalidReferenceIndex(index));
        }

        Ok(Self(index))
    }

    /// Return the index of a `ReferenceUnlock`.
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ReferenceUnlock {
    type Error = ValidationError;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}
