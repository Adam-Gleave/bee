// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the FPC statement payload.

mod conflicts;
mod timestamps;

pub use conflicts::{Conflict, Conflicts};
pub use timestamps::{Timestamp, Timestamps};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// Payload describing opinions on conflicts and timestamps of messages.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FpcPayload {
    /// Version of the FPC statement payload.
    version: u8,
    /// Collection of opinions on conflicting transactions.
    conflicts: Conflicts,
    /// Collection of opinions on message timestamps.
    timestamps: Timestamps,
}

impl FpcPayload {
    /// The payload kind of an `FpcPayload` (Using the same type as GoShimmer here).
    pub const KIND: u32 = 3;

    /// Returns a new `FpcPayloadBuilder` in order to build an `FpcPayload`.
    pub fn builder() -> FpcPayloadBuilder {
        FpcPayloadBuilder::new()
    }
}

impl Packable for FpcPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.conflicts.packed_len()
            + self.timestamps.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.version.pack(writer)?;
        self.conflicts.pack(writer)?;
        self.timestamps.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let version = u8::unpack_inner::<R, CHECK>(reader)?;
        let conflicts = Conflicts::unpack_inner::<R, CHECK>(reader)?;
        let timestamps = Timestamps::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            version,
            conflicts,
            timestamps,
        })
    }
}

/// A builder to build an `FpcPayload`.
pub struct FpcPayloadBuilder {
    version: Option<u8>,
    conflicts: Conflicts,
    timestamps: Timestamps,
}

impl FpcPayloadBuilder {
    /// Creates a new `FpcPayloadBuilder`.
    pub fn new() -> Self {
        Self {
            version: None,
            conflicts: Default::default(),
            timestamps: Default::default(),
        }
    }

    /// Adds a version number to the `FpcPayloadBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    /// Adds a collection of conflicts to the `FpcPayloadBuilder`.
    pub fn with_conflicts(mut self, conflicts: Conflicts) -> Self {
        self.conflicts = conflicts;
        self
    }

    /// Adds a collection of timestamps to the `FpcPayloadBuilder`.
    pub fn with_timestamps(mut self, timestamps: Timestamps) -> Self {
        self.timestamps = timestamps;
        self
    }

    /// Finishes an `FpcPayloadBuilder` into an `FpcPayload`.
    pub fn finish(self) -> Result<FpcPayload, Error> {
        let version = self.version.ok_or(Error::MissingField("version"))?;

        Ok(FpcPayload {
            version,
            conflicts: self.conflicts,
            timestamps: self.timestamps,
        })
    }
}
