// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{context::{ObjectType, VoteContext}, error::Error, opinion::{Opinion, QueriedOpinions}};

use std::{collections::HashMap, time::Duration};

pub trait Voter {
    fn vote(id: String, object_type: ObjectType, initial_opinion: Opinion) -> Result<(), Error>;

    fn intermediate_opinion(id: String) -> Result<Opinion, Error>;

    //TODO events
}

/// Holds data about a voting round.
#[derive(Debug)]
pub struct RoundStats {
    /// Time taken to complete the round.
    duration: Duration,
    /// Random number used during the round.
    rand_used: f64,
    /// The `VoteContext`s upon which `Opinion`s were formed and queried.
    /// This does not contain the `VoteContext`s that were finalized/aborted during the round.
    active_vote_contexts: HashMap<String, VoteContext>,
    /// The `Opinion`s that were queried during the round, per the `OpinionGiver`.
    queried_opinions: Vec<QueriedOpinions>,
}

/// Holds data relating to a Finalized or Failed event.
pub struct OpinionEvent {
    /// ID of the conflict.
    id: String,
    /// `Opinion` about the conflict.
    opinion: Opinion,
    /// `VoteContext` with all relevant information about the conflict.
    context: VoteContext,
}
