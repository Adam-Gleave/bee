// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Fpc instance has no OpinionGiver provider function, initialise the `opinion_giver_fn` field")]
    FpcNoOpinionGiverFn,
    #[error("Fpc instance has no event sender, initialise the `tx` field")]
    FpcNoSender,
    #[error("No opinion givers are available")]
    NoOpinionGivers,
    #[error("Vote already ongoing for ID {0}")]
    VoteOngoing(String),
    #[error("Voting not found for ID {0}")]
    VotingNotFound(String),
}
