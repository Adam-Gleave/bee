// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{context::VoteContext, error::Error, opinion::OpinionGiver};

use std::{collections::{HashMap, HashSet, VecDeque}, default::Default, sync::Mutex, time::Duration};

pub struct Fpc<F>
where 
    F: Fn() -> Result<Box<dyn OpinionGiver>, Error>
{
    opinion_giver_fn: F,
    queue: Mutex<VecDeque<String>>,
    queue_set: Mutex<HashSet<String>>,
    contexts: Mutex<HashMap<String, VoteContext>>,
    params: FpcParameters,
    last_round_successful: bool,
    //TODO rng
}

impl<F> Fpc<F>
where
    F: Fn() -> Result<Box<dyn OpinionGiver>, Error> 
{
    pub fn new(opinion_giver_fn: F) -> Self {
        Self {
            opinion_giver_fn,
            queue: Mutex::new(VecDeque::new()),
            queue_set: Mutex::new(HashSet::new()),
            contexts: Mutex::new(HashMap::new()),
            params: Default::default(),
            last_round_successful: false,
        }
    }

    pub fn with_params(mut self, params: FpcParameters) -> Self {
        self.params = params;
        self
    }
}

pub struct FpcParameters {
    first_round_lower_bound: f64,
    first_round_upper_bound: f64,
    subsequent_rounds_lower_bound: f64, 
    subsequent_rounds_upper_bound: f64,
    query_sample_size: u32,
    finalization_threshold: u32,
    cooling_off_period: u32,
    max_rounds_per_vote_context: u32,
    query_timeout: Duration,
}

impl Default for FpcParameters {
    fn default() -> Self {
        Self {
            first_round_lower_bound: 0.67,
            first_round_upper_bound: 0.67,
            subsequent_rounds_lower_bound: 0.5,
            subsequent_rounds_upper_bound: 0.67,
            query_sample_size: 21,
            finalization_threshold: 10,
            cooling_off_period: 0,
            max_rounds_per_vote_context: 100,
            query_timeout: Duration::from_millis(6500),
        }
    }
}
