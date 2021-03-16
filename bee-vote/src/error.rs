// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No opinion givers are available")]
    NoOpinionGivers,
    #[error("Vote already ongoing for ID {0}")]
    VoteOngoing(String),
    #[error("Voting not found for ID {0}")]
    VotingNotFound(String),
}
