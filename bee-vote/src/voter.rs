// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{context::ObjectType, error::Error, opinion::Opinion};

pub trait Voter {
    fn vote(id: String, object_type: ObjectType, initial_opinion: Opinion) -> Result<(), Error>;

    fn intermediate_opinion(id: String) -> Result<Opinion, Error>;

    //TODO events
}
