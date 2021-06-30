// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageId;

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    Packable, VecPrefix,
};

use alloc::vec::Vec;
use core::{convert::Infallible, ops::Deref};

/// Provides a convenient collection of `Timestamp`s.
/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Default, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible, u32>)]
#[packable(unpack_error = UnpackPrefixError<Infallible, u32>)]
pub struct Timestamps {
    #[packable(wrapper = VecPrefix<Timestamp, u32>)]
    inner: Vec<Timestamp>,
}

impl Deref for Timestamps {
    type Target = [Timestamp];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamp {
    /// ID of the message that contains the timestamp.
    message_id: MessageId,
    /// The nodes opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl Timestamp {
    /// Returns the ID of message that contains the timestamp.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the nodes opinion value in a given round.
    pub fn opinion(&self) -> u8 {
        self.opinion
    }

    /// Returns the voting round number.
    pub fn round(&self) -> u8 {
        self.round
    }
}