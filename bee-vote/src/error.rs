// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Voting not found for ID {0}")]
    VotingNotFound(String),
}

