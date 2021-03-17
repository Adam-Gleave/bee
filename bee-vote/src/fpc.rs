// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use crate::{context::{ObjectType, VoteContext}, error::Error, opinion::{self, Opinion, OpinionGiver, Opinions, QueriedOpinions, QueryIds}};

use rand::prelude::*;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    default::Default,
    sync::{Arc, Mutex},
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
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
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
    F: Fn() -> Result<Vec<Box<dyn OpinionGiver>>, Error>,
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

    pub async fn query_opinions(&self) -> Result<Vec<QueriedOpinions>, Error> {
        let mut rng = thread_rng();
        let query_ids = self.vote_context_ids();

        if query_ids.conflict_ids.len() == 0 && query_ids.timestamp_ids.len() == 0 {
            return Ok(vec![]);
        }

        let opinion_givers = (self.opinion_giver_fn)()?;

        if opinion_givers.len() == 0 {
            return Err(Error::NoOpinionGivers);
        }

        let dist = rand::distributions::Uniform::new_inclusive(0, opinion_givers.len());
        let mut queries = vec![0u32; opinion_givers.len()];

        for _ in 0..self.params.query_sample_size {
            let index = rng.sample(dist);
            
            if let Some(selected_count) = queries.get_mut(index) {
                *selected_count = *selected_count + 1;
            }
        }

        let vote_map = Arc::new(Mutex::new(HashMap::<String, Opinions>::new()));
        let all_queried_opinions = Arc::new(Mutex::new(Vec::<QueriedOpinions>::new()));

        let mut futures = vec![];

        for (i, selected_count) in queries.iter().enumerate() {
            if *selected_count > 0 {
                let opinion_giver = &opinion_givers[i];

                futures.push(Self::do_query(
                    &query_ids,
                    vote_map.clone(),
                    all_queried_opinions.clone(),
                    opinion_giver, 
                    *selected_count,
                ));
            }
        }

        futures::future::join_all(futures).await;

        let mut contexts_guard = self.contexts.lock().unwrap();
        let votes_guard = vote_map.lock().unwrap();

        for (id, votes) in votes_guard.iter() {
            let mut liked_sum = 0.0;
            let mut voted_count = votes.len() as f64;
        
            for vote in votes.iter() {
                match vote {
                    Opinion::Unknown => { voted_count -= 1.0 },
                    Opinion::Like    => { liked_sum += 1.0 },
                    _ => {}
                } 
            }

            contexts_guard.get_mut(id).unwrap().round_completed();
            contexts_guard.get_mut(id).unwrap().set_liked(liked_sum / voted_count);
        }

        Ok(Arc::try_unwrap(all_queried_opinions).unwrap().into_inner().unwrap())
    }

    async fn do_query(
        query_ids: &QueryIds, 
        vote_map: Arc<Mutex<HashMap<String, Opinions>>>,
        all_queried_opinions: Arc<Mutex<Vec<QueriedOpinions>>>,
        opinion_giver: &Box<dyn OpinionGiver>, 
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
            opinions:  HashMap::new(),
            times_counted: selected_count,
        };

        let mut vote_map_guard = vote_map.lock().unwrap();

        for (i, id) in query_ids.conflict_ids.iter().enumerate() {
            let mut votes = vote_map_guard.get(id).map_or(Opinions::new(vec![]), |opt| opt.clone());

            for _ in 0..selected_count {
                votes.push(opinions[i]);
            }

            *queried_opinions.opinions.get_mut(id).unwrap() = opinions[i];
            
            if vote_map_guard.contains_key(id) {
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

            *queried_opinions.opinions.get_mut(id).unwrap() = opinions[i];
            
            if vote_map_guard.contains_key(id) {
                *vote_map_guard.get_mut(id).unwrap() = votes;
            } else {
                vote_map_guard.insert(id.to_string(), votes);
            }
        }

        all_queried_opinions.lock().unwrap().push(queried_opinions);
    } 

    fn vote_context_ids(&self) -> QueryIds {
        let context_guard = self.contexts.lock().unwrap();
        let mut conflict_ids = vec![];
        let mut timestamp_ids = vec![];

        for (id, context) in context_guard.iter() {
            match context.object_type() {
                ObjectType::Conflict => { conflict_ids.push(id.clone()); }
                ObjectType::Timestamp => { timestamp_ids.push(id.clone()); }
            }
        }

        QueryIds { conflict_ids, timestamp_ids }
    }

    fn rand_uniform_threshold(&self, rand: f64, lower_bound: f64, upper_bound: f64) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::LIKED_INITIAL, opinion::{Opinion, Opinions},
    };

    impl VoteContext {
        fn with_opinions(opinions: Opinions) -> Self {
            Self {
                id: "test".to_string(),
                object_type: ObjectType::Conflict,
                liked: LIKED_INITIAL,
                rounds: 0,
                opinions,
            }
        }
    }
    #[test]
    fn is_finalized() {
        let ctx = VoteContext::with_opinions(Opinions::new(
            vec![Opinion::Like, Opinion::Like, Opinion::Like, Opinion::Like, Opinion::Like],
        ));

        assert!(ctx.finalized(2, 2));
    }

    #[test]
    fn is_not_finalized() {
        let ctx = VoteContext::with_opinions(Opinions::new(
            vec![Opinion::Like, Opinion::Like, Opinion::Like, Opinion::Like, Opinion::Dislike],
        ));

        assert!(!ctx.finalized(2, 2));
    }

    #[test]
    fn last_opinion() {
        let ctx = VoteContext::with_opinions(Opinions::new(
            vec![Opinion::Like, Opinion::Like, Opinion::Like, Opinion::Like],
        ));

        assert_eq!(ctx.last_opinion(), Some(Opinion::Like));

        let ctx = VoteContext::with_opinions(Opinions::new(
            vec![Opinion::Like, Opinion::Like, Opinion::Like, Opinion::Dislike],
        ));

        assert_eq!(ctx.last_opinion(), Some(Opinion::Dislike));
    }

    #[test]
    fn prohibit_multiple_votes() {
        let opinion_giver_fn = || { Err(Error::NoOpinionGivers) };

        let voter = Fpc { 
            opinion_giver_fn: Box::new(opinion_giver_fn),
            queue: Mutex::new(Queue::new()),
            contexts: Mutex::new(HashMap::new()),
            params: Default::default(),
            last_round_successful: false,
        };

        let id = "test".to_string();
        assert!(voter.vote(id.clone(), ObjectType::Conflict, Opinion::Like).is_ok());
        assert!(matches!(voter.vote(id.clone(), ObjectType::Conflict, Opinion::Like), Err(Error::VoteOngoing(_))));
    }
}
