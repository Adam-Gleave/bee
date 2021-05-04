// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Functionality for performing an FPC vote.

use crate::{
    context::{VoteContext, VoteObject},
    error::Error,
    events::{Event, OpinionEvent, RoundStats},
    opinion::{Opinion, OpinionGiver, Opinions, QueriedOpinions, QueryObjects},
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

/// Stores `VoteContext`s in a queue, and provides a HashSet for quick lookup.
#[derive(Debug)]
struct Queue {
    /// Queue of all `VoteContext`s
    queue: VecDeque<VoteContext>,
    /// `HashSet` of IDs, for quick lookup.
    queue_set: HashSet<VoteObject>,
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
    pub fn contains(&self, value: &VoteObject) -> bool {
        self.queue_set.contains(value)
    }

    /// Push a new `VoteContext` to the end of the queue.
    pub fn push(&mut self, context: VoteContext) {
        self.queue_set.insert(context.object());
        self.queue.push_back(context);
    }

    /// Pop a `VoteContext` from the front of the queue.
    pub fn pop(&mut self) -> Option<VoteContext> {
        let context = self.queue.pop_front()?;
        self.queue_set.remove(&context.object());

        Some(context)
    }
}

/// Builder pattern struct for instantiating `Fpc`s.
pub struct FpcBuilder<F>
where
    F: Fn() -> Vec<Box<dyn OpinionGiver>>,
{
    tx: Option<Sender<Event>>,
    opinion_giver_fn: Option<F>,
    first_round_lower_bound: f64,
    first_round_upper_bound: f64,
    subsequent_rounds_lower_bound: f64,
    subsequent_rounds_upper_bound: f64,
    ending_rounds_fixed_threshold: f64,
    query_sample_size: u32,
    total_rounds_finalization: u32,
    total_rounds_fixed: u32,
    cooling_off_period: u32,
    max_rounds_per_vote_context: u32,
    query_timeout_ms: u64,
    min_opinions_received: u32,
}

impl<F> Default for FpcBuilder<F>
where
    F: Fn() -> Vec<Box<dyn OpinionGiver>>,
{
    /// Initialise with default parameters.
    /// Note that the `tx` and `opinion_giver_fn` fields still need to be set before building.
    fn default() -> Self {
        Self {
            tx: None,
            opinion_giver_fn: None,
            first_round_lower_bound: 0.67,
            first_round_upper_bound: 0.67,
            subsequent_rounds_lower_bound: 0.5,
            subsequent_rounds_upper_bound: 0.67,
            ending_rounds_fixed_threshold: 0.5,
            query_sample_size: DEFAULT_SAMPLE_SIZE,
            total_rounds_finalization: 10,
            total_rounds_fixed: 3,
            cooling_off_period: 0,
            max_rounds_per_vote_context: 100,
            query_timeout_ms: 1500,
            min_opinions_received: 1,
        }
    }
}

impl<F> FpcBuilder<F>
where
    F: Fn() -> Vec<Box<dyn OpinionGiver>>,
{
    /// Provide a `Sender<Event>` to the builder, so that the user may receive events as the voting proceeds.
    pub fn with_tx(mut self, tx: Sender<Event>) -> Self {
        self.tx = Some(tx);
        self
    }

    /// Provide a closure to the builder that describes the `OpinionGivers` that will be used for voting.
    pub fn with_opinion_giver_fn(mut self, opinion_giver_fn: F) -> Self {
        self.opinion_giver_fn = Some(opinion_giver_fn);
        self
    }

    /// Provide upper and lower bounds for random opinion forming threshold, used on the first voting round.
    /// These bounds will be used to determine whether a `VoteContext` likes or dislikes a voting object.
    pub fn with_first_round_bounds(mut self, lower: f64, upper: f64) -> Self {
        self.first_round_lower_bound = lower;
        self.first_round_upper_bound = upper;
        self
    }

    /// Provide upper and lower bounds for random opinion forming threshold, used on subsequent voting rounds.
    /// These bounds will be used to determine whether a `VoteContext` likes or dislikes a voting object.
    pub fn with_subsequent_rounds_bounds(mut self, lower: f64, upper: f64) -> Self {
        self.subsequent_rounds_lower_bound = lower;
        self.subsequent_rounds_upper_bound = upper;
        self
    }

    pub fn with_ending_rounds_fixed_threshold(mut self, threshold: f64) -> Self {
        self.ending_rounds_fixed_threshold = threshold;
        self
    }

    /// Provide a query sample size.
    /// This is used to define the number of `Opinion`s to query on each voting round.
    pub fn with_query_sample_size(mut self, sample_size: u32) -> Self {
        self.query_sample_size = sample_size;
        self
    }

    /// Provide a finalization threshold.
    /// This is used to define the number of voting rounds in which a `VoteContext`s opinion must stay constant for.
    pub fn with_finalization_rounds(mut self, rounds: u32) -> Self {
        self.total_rounds_finalization = rounds;
        self
    }

    /// Provide a number of fixed rounds.
    /// "Fixed rounds" are performed in the last `n` rounds of a vote, and consist of a fixed opinion threshold, rather
    /// than a random number between two bounds.
    pub fn with_fixed_rounds(mut self, rounds: u32) -> Self {
        self.total_rounds_fixed = rounds;
        self
    }

    /// Provide a cool-off period.
    /// This is used to define the number of voting rounds in which to skip any finalization checks.
    pub fn with_cooling_off_period(mut self, period: u32) -> Self {
        self.cooling_off_period = period;
        self
    }

    /// Define the maximum number of rounds to execute before aborting the vote (if not finalized).
    pub fn with_max_rounds(mut self, max: u32) -> Self {
        self.max_rounds_per_vote_context = max;
        self
    }

    /// Define the minimum number of opinions received in a voting round for it to be considered valid.
    pub fn with_min_opinions_received(mut self, min_opinions_received: u32) -> Self {
        self.min_opinions_received = min_opinions_received;
        self
    }

    /// Provide a timeout in which to query an opinion giver. If the query does not complete, an error will occur.
    pub fn with_query_timeout_ms(mut self, query_timeout_ms: u64) -> Self {
        self.query_timeout_ms = query_timeout_ms;
        self
    }

    /// Instantiate a new `Fpc` struct using parameters given by the `FpcBuilder`.
    /// Note: this will panic if `tx` or `opinion_giver_fn` are not defined.
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
            ending_rounds_fixed_threshold: self.ending_rounds_fixed_threshold,
            query_sample_size: self.query_sample_size,
            total_rounds_finalization: self.total_rounds_finalization,
            total_rounds_fixed: self.total_rounds_fixed,
            cooling_off_period: self.cooling_off_period,
            max_rounds_per_vote_context: self.max_rounds_per_vote_context,
            query_timeout: Duration::from_millis(self.query_timeout_ms),
            min_opinions_received: self.min_opinions_received,
        })
    }
}

/// Contains all instance information about a vote, including all `VoteContext`s, a queue of contexts
/// to be added to the vote in the next round, and RNG paramaters.
#[derive(Debug)]
pub struct Fpc<F>
where
    F: Fn() -> Vec<Box<dyn OpinionGiver>>,
{
    /// `Sender` for transmitting voting events through a channel.
    tx: Sender<Event>,
    /// Closure that describes the `OpinionGiver`s used in the vote.
    opinion_giver_fn: F,
    /// `Queue` of `VoteContext`s to be added to the next voting round.
    queue: RwLock<Queue>,
    /// Map of `VoteContext` IDs to contexts.
    /// Contains all `VoteContext`s that are participating in this voting round.
    contexts: RwLock<HashMap<VoteObject, VoteContext>>,
    /// Indicates whether the last round completed without error or other failure.
    /// These will be indicated through `Error` or `Failed` events.
    last_round_successful: AtomicBool,
    /// Lower bound for random opinion forming threshold, used on the first voting round.
    /// These bounds will be used to determine whether a `VoteContext` likes or dislikes a voting object.
    first_round_lower_bound: f64,
    /// Upper bound for random opinion forming threshold, used on the first voting round.
    first_round_upper_bound: f64,
    /// Lower bound for random opinion forming threshold, used on subsequent voting rounds.
    subsequent_rounds_lower_bound: f64,
    /// Upper bound for random opinion forming threshold, used on subsequent voting rounds.
    subsequent_rounds_upper_bound: f64,
    /// Fixed threshold used in the ending rounds of a vote (defined by `total_rounds_fixed`).
    ending_rounds_fixed_threshold: f64,
    /// Number of `Opinion`s to query on each voting round.
    query_sample_size: u32,
    /// Number of voting rounds in which a `VoteContext`s opinion must stay constant for.
    total_rounds_finalization: u32,
    /// Number of rounds at the end of the vote that use the fixed threshold (rather than random).
    total_rounds_fixed: u32,
    /// Number of voting rounds in which to skip any finalization checks.
    cooling_off_period: u32,
    /// Maximum number of rounds to execute before aborting the vote (if not finalized).
    max_rounds_per_vote_context: u32,
    /// Maximum time before aborting a query.
    query_timeout: Duration,
    /// Minimum opinions to receive in order to consider a voting round valid.
    min_opinions_received: u32,
}

impl<F> Fpc<F>
where
    F: Fn() -> Vec<Box<dyn OpinionGiver>>,
{
    /// Add a `VoteContext` to the queue for the next round, providing a voting object and an initial opinion
    /// of the context.
    /// This can fail if there is already a vote ongoing for this ID.
    pub async fn vote(&self, object: VoteObject, initial_opinion: Opinion) -> Result<(), Error> {
        let mut queue_guard = self.queue.write().await;
        let context_guard = self.contexts.read().await;

        if queue_guard.contains(&object) {
            return Err(Error::VoteOngoing(object));
        }

        if context_guard.contains_key(&object) {
            return Err(Error::VoteOngoing(object));
        }

        queue_guard.push(VoteContext::new(object, initial_opinion));
        Ok(())
    }

    /// Return the most recent opinion on the given ID. If a `VoteContext` with the ID does not exist, returns None.
    pub async fn intermediate_opinion(&self, id: VoteObject) -> Option<Opinion> {
        if let Some(context) = self.contexts.read().await.get(&id) {
            context.last_opinion()
        } else {
            Some(Opinion::Unknown)
        }
    }

    /// Add a `VoteContext` to the queue, to participate on the voting for the next round.
    async fn enqueue(&self) {
        let mut queue_guard = self.queue.write().await;
        let mut context_guard = self.contexts.write().await;

        while let Some(context) = queue_guard.pop() {
            context_guard.insert(context.object(), context);
        }
    }

    /// Loop through all `VoteContext`s that are participating, and have them form an opinion on the voting object.
    async fn form_opinions(&self, rand: f64) {
        let mut context_guard = self.contexts.write().await;

        for context in context_guard.values_mut() {
            if context.is_new() {
                continue;
            }

            let (lower_bound, upper_bound) = self.calculate_round_thresholds(&context);

            // This will never fail, since we skip this loop if the context is new.
            if context.liked().unwrap() >= self.rand_uniform_threshold(rand, lower_bound, upper_bound) {
                context.add_opinion(Opinion::Like);
            } else {
                context.add_opinion(Opinion::Dislike);
            }
        }
    }

    /// Check if any `VoteContext`s have finalized opinions.
    /// If a context has finalized on an opinion, send an event down the channel and remove it from the voting pool.
    async fn finalize_opinions(&self) -> Result<(), Error> {
        let mut context_guard = self.contexts.write().await;
        let mut to_remove = vec![];

        for (object, context) in context_guard.iter() {
            // Check for a finalized vote, and send an event.
            if context.finalized(self.cooling_off_period, self.total_rounds_finalization) {
                self.tx
                    .send(Event::Finalized(OpinionEvent {
                        object: *object,
                        opinion: context.last_opinion().ok_or(Error::Unknown("No opinions found"))?,
                        context: context.clone(),
                    }))
                    .or(Err(Error::SendError))?;

                to_remove.push(*object);
                continue;
            }

            // Check for a failed vote, and send an event.
            if context.rounds() >= self.max_rounds_per_vote_context {
                self.tx
                    .send(Event::Failed(OpinionEvent {
                        object: *object,
                        opinion: context.last_opinion().ok_or(Error::Unknown("No opinions found"))?,
                        context: context.clone(),
                    }))
                    .or(Err(Error::SendError))?;

                to_remove.push(*object);
            }
        }

        // Remove any finalized/failed votes.
        for object in to_remove {
            context_guard.remove(&object);
        }

        Ok(())
    }

    /// Perform the voting round, with a given threshold (between 0 and 1).
    /// This threshold is used to generate opinions on the voting object.
    ///
    /// For each `VoteContext` in the voting pool, a random number is generated within the range
    /// given on initialisation of the `Fpc` struct, and compared to the threshold to generate a
    /// `Like` or `Dislike` opinion.
    pub async fn do_round(&self, rand: f64) -> Result<(), Error> {
        let start = Instant::now();
        self.enqueue().await;

        if self.last_round_successful.load(Ordering::Relaxed) {
            self.form_opinions(rand).await;
            self.finalize_opinions().await?;
        }

        let queried_opinions = self.query_opinions().await?;
        self.last_round_successful.store(true, Ordering::Relaxed);

        let round_stats = RoundStats {
            duration: start.elapsed(),
            rand_used: rand,
            vote_contexts: self.contexts.read().await.clone(),
            queried_opinions,
        };

        self.tx
            .send(Event::RoundExecuted(round_stats))
            .or(Err(Error::SendError))?;

        Ok(())
    }

    /// Select a number of `OpinionGiver`s and query them for opinions.
    async fn query_opinions(&self) -> Result<Vec<QueriedOpinions>, Error> {
        let query_objects = self.vote_context_objects().await;
        if query_objects.conflict_objects.is_empty() && query_objects.timestamp_objects.is_empty() {
            return Ok(vec![]);
        }

        // Create opinion givers, validating that there is at last one.
        let mut opinion_givers = (self.opinion_giver_fn)();
        if opinion_givers.is_empty() {
            return Err(Error::NoOpinionGivers);
        }

        let selected_opinions = self.select_opinions(&opinion_givers);
        let (vote_map, all_queried_opinions) = self
            .run_all_queries(&mut opinion_givers, query_objects, &selected_opinions)
            .await;

        self.calculate_liked_percentages(vote_map).await;

        Ok(all_queried_opinions)
    }

    /// Select opinions for voting using rng.
    fn select_opinions(&self, opinion_givers: &[Box<dyn OpinionGiver>]) -> Vec<u32> {
        let mut rng = thread_rng();
        let dist = rand::distributions::Uniform::new(0, opinion_givers.len());

        let mut selected_opinions = vec![0u32; opinion_givers.len()];

        for _ in 0..self.query_sample_size {
            let query_idx = rng.sample(dist);

            if let Some(opinion_select_count) = selected_opinions.get_mut(query_idx) {
                *opinion_select_count += 1;
            }
        }

        selected_opinions
    }

    /// Run all query futures, returning a map of `VoteObject`s to `Opinions`,
    /// and a list of `QueriedOpinions` that describes all `Opinion`s generated by an `OpinionGiver`.
    async fn run_all_queries(
        &self,
        opinion_givers: &mut Vec<Box<dyn OpinionGiver>>,
        query_objects: QueryObjects,
        selected_opinions: &[u32],
    ) -> (HashMap<VoteObject, Opinions>, Vec<QueriedOpinions>) {
        let vote_map = Arc::new(RwLock::new(HashMap::<VoteObject, Opinions>::new()));
        let all_queried_opinions = Arc::new(RwLock::new(Vec::<QueriedOpinions>::new()));
        let mut query_futures = vec![];

        for (opinion_giver, opinion_select_count) in opinion_givers.iter_mut().zip(selected_opinions.iter()) {
            if *opinion_select_count > 0 {
                query_futures.push(timeout(
                    self.query_timeout,
                    Self::do_query(
                        &query_objects,
                        vote_map.clone(),
                        all_queried_opinions.clone(),
                        opinion_giver,
                        *opinion_select_count,
                    ),
                ));
            }
        }

        futures::future::join_all(query_futures).await;

        (
            Arc::try_unwrap(vote_map).unwrap().into_inner(),
            Arc::try_unwrap(all_queried_opinions).unwrap().into_inner(),
        )
    }

    /// Run a query on a given `OpinionGiver`, to generate opinions on the voting object.
    async fn do_query(
        query_ids: &QueryObjects,
        vote_map: Arc<RwLock<HashMap<VoteObject, Opinions>>>,
        all_queried_opinions: Arc<RwLock<Vec<QueriedOpinions>>>,
        opinion_giver: &mut Box<dyn OpinionGiver>,
        opinion_select_count: u32,
    ) {
        // Get opinions from the `OpinionGiver`.
        let opinions = if let Ok(opinions) = opinion_giver.query(query_ids) {
            // Verify that the number of resulting opinions equals the number of object queries being made.
            if opinions.len() != query_ids.conflict_objects.len() + query_ids.timestamp_objects.len() {
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
            times_counted: opinion_select_count,
        };

        let mut vote_map_guard = vote_map.write().await;

        // Get opinions on a voting object and add to vote map.
        let mut query_voting_objects = |vote_objects: &Vec<VoteObject>| {
            for (vote_object, opinion) in vote_objects.iter().zip(opinions.iter()) {
                let mut votes = vote_map_guard
                    .get(vote_object)
                    .map_or_else(|| Opinions::new(vec![]), |votes| votes.clone());

                for _ in 0..opinion_select_count {
                    votes.push(*opinion);
                }

                queried_opinions.opinions.insert(*vote_object, *opinion);

                if let Some(opinions) = vote_map_guard.get_mut(vote_object) {
                    *opinions = votes;
                } else {
                    vote_map_guard.insert(*vote_object, votes);
                }
            }
        };

        query_voting_objects(&query_ids.conflict_objects);
        query_voting_objects(&query_ids.timestamp_objects);

        all_queried_opinions.write().await.push(queried_opinions);
    }

    /// Calculate liked percentage for each vote context.
    async fn calculate_liked_percentages(&self, vote_map: HashMap<VoteObject, Opinions>) {
        let mut contexts_guard = self.contexts.write().await;

        for (object, votes) in vote_map.iter() {
            let mut liked_sum = 0.0;
            let mut voted_count = votes.len() as u32;

            for vote in votes.iter() {
                match vote {
                    Opinion::Unknown => voted_count -= 1,
                    Opinion::Like => liked_sum += 1.0,
                    _ => {}
                }
            }

            // This should never happen – there should always be a context for a given vote.
            contexts_guard.get_mut(object).unwrap().round_completed();

            // Make sure enough opinions were received for the round to be considered valid.
            if voted_count < self.min_opinions_received {
                continue;
            }

            contexts_guard
                .get_mut(object)
                .unwrap()
                .set_liked(liked_sum / voted_count as f64);
        }
    }

    /// Get the IDs of all `VoteContext`s currently in the voting pool.
    async fn vote_context_objects(&self) -> QueryObjects {
        let context_guard = self.contexts.read().await;
        let mut conflict_objects = vec![];
        let mut timestamp_objects = vec![];

        for context in context_guard.values() {
            match context.object() {
                VoteObject::Conflict(_) => {
                    conflict_objects.push(context.object());
                }
                VoteObject::Timestamp(_) => {
                    timestamp_objects.push(context.object());
                }
            }
        }

        QueryObjects {
            conflict_objects,
            timestamp_objects,
        }
    }

    /// Calculates and returns the bounds used to select the threshold at which the node will change its opinion.
    fn calculate_round_thresholds(&self, ctx: &VoteContext) -> (f64, f64) {
        if ctx.had_first_round() {
            (self.first_round_lower_bound, self.first_round_upper_bound)
        } else if ctx.had_fixed_round(
            self.cooling_off_period,
            self.total_rounds_finalization,
            self.total_rounds_fixed,
        ) {
            (self.ending_rounds_fixed_threshold, self.ending_rounds_fixed_threshold)
        } else {
            (self.subsequent_rounds_lower_bound, self.subsequent_rounds_upper_bound)
        }
    }

    fn rand_uniform_threshold(&self, rand: f64, lower_bound: f64, upper_bound: f64) -> f64 {
        lower_bound + rand * (upper_bound - lower_bound)
    }
}