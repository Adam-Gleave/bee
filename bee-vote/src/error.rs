// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// User error: Fpc struct not built properly – no `opinion_giver_fn`.
    #[error("Fpc instance has no OpinionGiver provider function, initialise the `opinion_giver_fn` field")]
    FpcNoOpinionGiverFn,
    /// User error: Fpc struct not built properly – no `tx` for sending events.
    #[error("Fpc instance has no event sender, initialise the `tx` field")]
    FpcNoSender,
    /// Vote context has no `OpinionGiver`s available. 
    #[error("No opinion givers are available")]
    NoOpinionGivers,
    /// Vote is already ongoing.
    #[error("Vote already ongoing for ID {0}")]
    VoteOngoing(String),
    /// No vote found.
    #[error("Voting not found for ID {0}")]
    VotingNotFound(String),
}
