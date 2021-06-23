// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::payload::transaction::TransactionId;

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    Packable, VecPrefix,
};

use core::{convert::Infallible, ops::Deref};

/// Provides a convenient collection of `Conflict`s.
/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Default, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible, u32>)]
#[packable(unpack_error = UnpackPrefixError<Infallible, u32>)]
pub struct Conflicts {
    #[packable(wrapper = VecPrefix<Conflict, u32>)]
    inner: Vec<Conflict>,
}

impl Deref for Conflicts {
    type Target = [Conflict];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Conflict {
    /// ID of the conflicting transaction.
    transaction_id: TransactionId,
    /// The nodes opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl Conflict {
    /// Returns the ID of the conflicting transaction.
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
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
