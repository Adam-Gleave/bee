// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{
    context::{ObjectType, VoteContext},
    error::Error,
    events::{Event, OpinionEvent, RoundStats},
    opinion::{Opinion, OpinionGiver, Opinions, QueriedOpinions, QueryIds},
};

use flume::Sender;
use rand::prelude::*;
use tokio::{sync::RwLock, time::timeout};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    default::Default,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub const DEFAULT_SAMPLE_SIZE: u32 = 21;

/// Stores `VoteContext`s in a queue, and provides a HashMap for quick lookup.
#[derive(Debug)]
struct Queue {
    /// Queue of all `VoteContext`s
    queue: VecDeque<VoteContext>,
    /// `HashSet` of IDs, for quick lookup.
    queue_set: HashSet<String>,
}

impl Queue {
    /// Construct a new, empty `Queue`.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            queue_set: HashSet::new(),
        }
    }

    /// Look up a `VoteContext` ID and determine if it is in the queue.
    pub fn contains(&self, value: &str) -> bool {
        self.queue_set.contains(value)
    }

    /// Push a new `VoteContext` to the end of the queue.
    pub fn push(&mut self, context: VoteContext) {
        self.queue_set.insert(context.id());
        self.queue.push_back(context);
    }

    /// Pop a `VoteContext` from the front of the queue.
    pub fn pop(&mut self) -> Option<VoteContext> {
        let context = self.queue.pop_front()?;
        self.queue_set.remove(&context.id());

        Some(context)
    }
}

pub struct FpcBuilder<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    tx: Option<Sender<Event>>,
    opinion_giver_fn: Option<F>,
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

impl<F> Default for FpcBuilder<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    fn default() -> Self {
        Self {
            tx: None,
            opinion_giver_fn: None,
            first_round_lower_bound: 0.67,
            first_round_upper_bound: 0.67,
            subsequent_rounds_lower_bound: 0.5,
            subsequent_rounds_upper_bound: 0.67,
            query_sample_size: DEFAULT_SAMPLE_SIZE,
            finalization_threshold: 10,
            cooling_off_period: 0,
            max_rounds_per_vote_context: 100,
            query_timeout: Duration::from_millis(6500),
        }
    }
}

impl<F> FpcBuilder<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    pub fn with_tx(mut self, tx: Sender<Event>) -> Self {
        self.tx = Some(tx);
        self
    }

    pub fn with_opinion_giver_fn(mut self, opinion_giver_fn: F) -> Self {
        self.opinion_giver_fn = Some(opinion_giver_fn);
        self
    }

    pub fn with_first_round_bounds(mut self, lower: f64, upper: f64) -> Self {
        self.first_round_lower_bound = lower;
        self.first_round_upper_bound = upper;
        self
    }

    pub fn with_subsequent_rounds_bounds(mut self, lower: f64, upper: f64) -> Self {
        self.subsequent_rounds_lower_bound = lower;
        self.subsequent_rounds_upper_bound = upper;
        self
    }

    pub fn with_query_sample_size(mut self, sample_size: u32) -> Self {
        self.query_sample_size = sample_size;
        self
    }

    pub fn with_finalization_threshold(mut self, threshold: u32) -> Self {
        self.finalization_threshold = threshold;
        self
    }

    pub fn with_cooling_off_period(mut self, period: u32) -> Self {
        self.cooling_off_period = period;
        self
    }

    pub fn with_max_rounds(mut self, max: u32) -> Self {
        self.max_rounds_per_vote_context = max;
        self
    }

    pub fn build(self) -> Result<Fpc<F>, Error> {
        Ok(Fpc {
            tx: self.tx.ok_or(Error::FpcNoSender)?,
            opinion_giver_fn: self.opinion_giver_fn.ok_or(Error::FpcNoOpinionGiverFn)?,
            queue: RwLock::new(Queue::new()),
            contexts: RwLock::new(HashMap::new()),
            last_round_successful: AtomicBool::new(false),
            first_round_lower_bound: self.first_round_lower_bound,
            first_round_upper_bound: self.first_round_lower_bound,
            subsequent_rounds_lower_bound: self.subsequent_rounds_lower_bound,
            subsequent_rounds_upper_bound: self.subsequent_rounds_upper_bound,
            query_sample_size: self.query_sample_size,
            finalization_threshold: self.finalization_threshold,
            cooling_off_period: self.cooling_off_period,
            max_rounds_per_vote_context: self.max_rounds_per_vote_context,
            query_timeout: Duration::from_millis(6500),
        })
    }
}

#[derive(Debug)]
pub struct Fpc<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    tx: Sender<Event>,
    opinion_giver_fn: F,
    queue: RwLock<Queue>,
    contexts: RwLock<HashMap<String, VoteContext>>,
    last_round_successful: AtomicBool,
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

impl<F> Fpc<F>
where
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
{
    pub async fn vote(&self, id: String, object_type: ObjectType, initial_opinion: Opinion) -> Result<(), Error> {
        let mut queue_guard = self.queue.write().await;
        let context_guard = self.contexts.read().await;

        if queue_guard.contains(&id) {
            return Err(Error::VoteOngoing(id));
        }

        if context_guard.contains_key(&id) {
            return Err(Error::VoteOngoing(id));
        }

        queue_guard.push(VoteContext::new(id, object_type, initial_opinion));
        Ok(())
    }

    pub async fn intermediate_opinion(&self, id: String) -> Option<Opinion> {
        if let Some(context) = self.contexts.read().await.get(&id) {
            context.last_opinion()
        } else {
            Some(Opinion::Unknown)
        }
    }

    pub async fn enqueue(&self) {
        let mut queue_guard = self.queue.write().await;
        let mut context_guard = self.contexts.write().await;

        while let Some(context) = queue_guard.pop() {
            context_guard.insert(context.id(), context);
        }
    }

    pub async fn form_opinions(&self, rand: f64) {
        let mut context_guard = self.contexts.write().await;

        for context in context_guard.values_mut() {
            if context.is_new() {
                continue;
            }

            let (lower_bound, upper_bound) = if context.had_first_round() {
                (self.first_round_lower_bound, self.first_round_upper_bound)
            } else {
                (self.subsequent_rounds_lower_bound, self.subsequent_rounds_upper_bound)
            };

            if context.liked() >= self.rand_uniform_threshold(rand, lower_bound, upper_bound) {
                context.add_opinion(Opinion::Like);
            } else {
                context.add_opinion(Opinion::Dislike);
            }
        }
    }

    pub async fn finalize_opinions(&self) -> Result<(), Error> {
        let context_guard = self.contexts.read().await;
        let mut to_remove = vec![];

        for (id, context) in context_guard.iter() {
            if context.finalized(self.cooling_off_period, self.finalization_threshold) {
                self.tx
                    .send(Event::Finalized(OpinionEvent {
                        id: id.clone(),
                        opinion: context.last_opinion().ok_or(Error::Unknown("No opinions found"))?,
                        context: context.clone(),
                    }))
                    .or(Err(Error::SendError));

                to_remove.push(id.clone());
                continue;
            }

            if context.rounds() >= self.max_rounds_per_vote_context {
                self.tx
                    .send(Event::Failed(OpinionEvent {
                        id: id.clone(),
                        opinion: context.last_opinion().ok_or(Error::Unknown("No opinions found"))?,
                        context: context.clone(),
                    }))
                    .or(Err(Error::SendError))?;

                to_remove.push(id.clone());
            }
        }
        drop(context_guard);

        let mut context_guard = self.contexts.write().await;

        for id in to_remove {
            context_guard.remove(&id);
        }

        Ok(())
    }

    pub async fn do_round(&self, rand: f64) -> Result<(), Error> {
        let start = Instant::now();
        self.enqueue().await;

        if self.last_round_successful.load(Ordering::Relaxed) {
            self.form_opinions(rand).await;
            self.finalize_opinions().await;
        }

        let queried_opinions = self.query_opinions().await?;
        self.last_round_successful.store(true, Ordering::Relaxed);

        let round_stats = RoundStats {
            duration: start.elapsed(),
            rand_used: rand,
            vote_contexts: self.contexts.read().await.clone(),
            queried_opinions,
        };

        self.tx.send(Event::RoundExecuted(round_stats)).or(Err(Error::SendError))?;

        Ok(())
    }

    pub async fn query_opinions(&self) -> Result<Vec<QueriedOpinions>, Error> {
        let mut rng = thread_rng();
        let query_ids = self.vote_context_ids().await;

        if query_ids.conflict_ids.is_empty() && query_ids.timestamp_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut opinion_givers = (self.opinion_giver_fn)()?;

        if opinion_givers.is_empty() {
            return Err(Error::NoOpinionGivers);
        }

        let dist = rand::distributions::Uniform::new(0, opinion_givers.len());
        let mut queries = vec![0u32; opinion_givers.len()];

        for _ in 0..self.query_sample_size {
            let index = rng.sample(dist);

            if let Some(selected_count) = queries.get_mut(index) {
                *selected_count += 1;
            }
        }

        let vote_map = Arc::new(RwLock::new(HashMap::<String, Opinions>::new()));
        let all_queried_opinions = Arc::new(RwLock::new(Vec::<QueriedOpinions>::new()));

        let mut futures = vec![];

        for (i, opinion_giver) in opinion_givers.iter_mut().enumerate() {
            // This should never panic, since `queries.len()` == `opinion_givers.len()`
            let selected_count = queries.get(i).unwrap();

            if *selected_count > 0 {
                futures.push(timeout(
                    self.query_timeout,
                    Self::do_query(
                        &query_ids,
                        vote_map.clone(),
                        all_queried_opinions.clone(),
                        opinion_giver,
                        *selected_count,
                    ),
                ));
            }
        }

        futures::future::join_all(futures).await;

        let mut contexts_guard = self.contexts.write().await;
        let votes_guard = vote_map.read().await;

        for (id, votes) in votes_guard.iter() {
            let mut liked_sum = 0.0;
            let mut voted_count = votes.len() as f64;

            for vote in votes.iter() {
                match vote {
                    Opinion::Unknown => voted_count -= 1.0,
                    Opinion::Like => liked_sum += 1.0,
                    _ => {}
                }
            }

            // This should never happen – there should always be a context for a given vote.
            contexts_guard.get_mut(id).unwrap().round_completed();

            if voted_count == 0.0 {
                continue;
            }

            contexts_guard.get_mut(id).unwrap().set_liked(liked_sum / voted_count);
        }

        // This should never fail – all futures are completed, so only one reference remains.
        Ok(Arc::try_unwrap(all_queried_opinions).unwrap().into_inner())
    }

    async fn do_query(
        query_ids: &QueryIds,
        vote_map: Arc<RwLock<HashMap<String, Opinions>>>,
        all_queried_opinions: Arc<RwLock<Vec<QueriedOpinions>>>,
        opinion_giver: &mut Box<dyn OpinionGiver>,
        selected_count: u32,
    ) {
        let opinions = opinion_giver.query(query_ids);

        let opinions = if let Ok(opinions) = opinions {
            if opinions.len() != query_ids.conflict_ids.len() + query_ids.timestamp_ids.len() {
                return;
            } else {
                opinions
            }
        } else {
            return;
        };

        let mut queried_opinions = QueriedOpinions {
            opinion_giver_id: opinion_giver.id().to_string(),
            opinions: HashMap::new(),
            times_counted: selected_count,
        };

        let mut vote_map_guard = vote_map.write().await;

        for (i, id) in query_ids.conflict_ids.iter().enumerate() {
            let mut votes = vote_map_guard.get(id).map_or(Opinions::new(vec![]), |opt| opt.clone());

            for _ in 0..selected_count {
                votes.push(opinions[i]);
            }

            queried_opinions.opinions.insert(id.to_string(), opinions[i]);

            if vote_map_guard.contains_key(id) {
                // This will never fail – the key exists.
                *vote_map_guard.get_mut(id).unwrap() = votes;
            } else {
                vote_map_guard.insert(id.to_string(), votes);
            }
        }

        for (i, id) in query_ids.timestamp_ids.iter().enumerate() {
            let mut votes = vote_map_guard.get(id).map_or(Opinions::new(vec![]), |opt| opt.clone());

            for _ in 0..selected_count {
                votes.push(opinions[i]);
            }

            queried_opinions.opinions.insert(id.to_string(), opinions[i]);

            if vote_map_guard.contains_key(id) {
                // This will never fail - the key exists.
                *vote_map_guard.get_mut(id).unwrap() = votes;
            } else {
                vote_map_guard.insert(id.to_string(), votes);
            }
        }

        all_queried_opinions.write().await.push(queried_opinions);
    }

    async fn vote_context_ids(&self) -> QueryIds {
        let context_guard = self.contexts.read().await;
        let mut conflict_ids = vec![];
        let mut timestamp_ids = vec![];

        for (id, context) in context_guard.iter() {
            match context.object_type() {
                ObjectType::Conflict => {
                    conflict_ids.push(id.clone());
                }
                ObjectType::Timestamp => {
                    timestamp_ids.push(id.clone());
                }
            }
        }

        QueryIds {
            conflict_ids,
            timestamp_ids,
        }
    }

    fn rand_uniform_threshold(&self, rand: f64, lower_bound: f64, upper_bound: f64) -> f64 {
        lower_bound + rand * (upper_bound - lower_bound)
    }
}

