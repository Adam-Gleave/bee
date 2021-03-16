// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{context::{self, ObjectType, VoteContext}, error::Error, opinion::{Opinion, OpinionGiver}};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    default::Default,
    sync::Mutex,
    time::Duration,
};

struct Queue {
    queue: VecDeque<VoteContext>,
    queue_set: HashSet<String>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            queue_set: HashSet::new(),
        }
    }

    pub fn contains(&self, value: &String) -> bool {
        self.queue_set.contains(value)
    }

    pub fn push(&mut self, context: VoteContext) {
        self.queue_set.insert(context.id());
        self.queue.push_back(context);
    }

    pub fn pop(&mut self) -> Option<VoteContext> {
        let context = self.queue.pop_front()?;
        self.queue_set.remove(&context.id());

        Some(context)
    }
}

pub struct Fpc<F>
where
    F: Fn() -> Result<Box<dyn OpinionGiver>, Error>,
{
    opinion_giver_fn: F,
    queue: Mutex<Queue>,
    contexts: Mutex<HashMap<String, VoteContext>>,
    params: FpcParameters,
    last_round_successful: bool,
    // TODO rng
}

impl<F> Fpc<F>
where
    F: Fn() -> Result<Box<dyn OpinionGiver>, Error>,
{
    pub fn new(opinion_giver_fn: F) -> Self {
        Self {
            opinion_giver_fn,
            queue: Mutex::new(Queue::new()),
            contexts: Mutex::new(HashMap::new()),
            params: Default::default(),
            last_round_successful: false,
        }
    }

    pub fn with_params(mut self, params: FpcParameters) -> Self {
        self.params = params;
        self
    }

    pub fn vote(&self, id: String, object_type: ObjectType, initial_opinion: Opinion) -> Result<(), Error> {
        let mut queue_guard = self.queue.lock().unwrap();
        let context_guard = self.contexts.lock().unwrap();

        if queue_guard.contains(&id) {
            return Err(Error::VoteOngoing(id));
        }

        if context_guard.contains_key(&id) {
            return Err(Error::VoteOngoing(id));
        }

        queue_guard.push(VoteContext::new(id, object_type, initial_opinion));
        Ok(())
    }

    pub fn intermediate_opinion(&self, id: String) -> Result<Opinion, Error> {
        if let Some(context) = self.contexts.lock().unwrap().get(&id) {
            Ok(context.last_opinion().unwrap())
        } else {
            Err(Error::VotingNotFound(id))
        }
    }

    pub fn enqueue(&self) {
        let mut queue_guard = self.queue.lock().unwrap();
        let mut context_guard = self.contexts.lock().unwrap();

        while let Some(context) = queue_guard.pop() {
            context_guard.insert(context.id(), context);
        }
    }

    pub fn form_opinions(&self, rand: f64) {
        let mut context_guard = self.contexts.lock().unwrap();

        for context in context_guard.values_mut() {
            if context.is_new() {
                continue;
            }

            let (lower_bound, upper_bound) = if context.had_first_round() {
                (self.params.first_round_lower_bound, self.params.first_round_upper_bound)
            } else {
                (self.params.subsequent_rounds_lower_bound, self.params.subsequent_rounds_upper_bound)
            };

            if context.liked() >= self.rand_uniform_threshold(rand, lower_bound, upper_bound) {
                context.add_opinion(Opinion::Like);
            } else {
                context.add_opinion(Opinion::Dislike);
            }
        }
    }

    pub fn finalize_opinions(&self) {
        let mut context_guard = self.contexts.lock().unwrap();
        let mut to_remove = vec![];

        for (id, context) in context_guard.iter() {
            if context.finalized(self.params.cooling_off_period, self.params.finalization_threshold) {
                //TODO fire finalized event
                to_remove.push(id.clone());
                continue;
            }

            if context.rounds() >= self.params.max_rounds_per_vote_context {
                //TODO fire failed event
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            context_guard.remove(&id);
        }        
    }

    pub fn rand_uniform_threshold(&self, rand: f64, lower_bound: f64, upper_bound: f64) -> f64 {
        lower_bound + rand * (upper_bound - lower_bound)
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
