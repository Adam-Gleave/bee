// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use bee_vote::{context::ObjectType, error::Error, events::Event, fpc::{Fpc, FpcParameters}, opinion::{Opinion, Opinions, OpinionGiver, QueryIds}};

use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[derive(Clone)]
struct MockOpinionGiver {
    id: String,
    round: u32,
    round_replies: Vec<Opinions>,
}

impl OpinionGiver for MockOpinionGiver {
    fn query(&mut self, _: &QueryIds) -> Result<Opinions, Error> {
        if self.round as usize >= self.round_replies.len() {
            return Ok(self.round_replies.last().unwrap().clone());
        }

        let opinions = self.round_replies.get(self.round as usize).unwrap().clone();
        self.round += 1;

        Ok(opinions)
    }
    
    fn id(&self) -> &str {
        &self.id
    }
}

fn random_id_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

#[tokio::test]
async fn finalized_event() {
    let mock = MockOpinionGiver {
        id: random_id_string(),
        round: 0,
        round_replies: vec![
            Opinions::new(vec![Opinion::Like]), 
            Opinions::new(vec![Opinion::Like]),
            Opinions::new(vec![Opinion::Like]),
            Opinions::new(vec![Opinion::Like]),
        ], 
    };

    let opinion_giver_fn = || -> Result<Vec<Box<dyn OpinionGiver>>, Error> {
        Ok(vec![Box::new(mock.clone())])
    };

    let id = String::from("test");
    
    let mut params = FpcParameters::default();
    params.finalization_threshold = 2;
    params.cooling_off_period = 2;
    params.query_sample_size = 1;

    let (tx, rx) = flume::unbounded();

    let voter = Fpc::new(opinion_giver_fn).with_params(params).with_tx(tx);    

    assert!(voter.vote(id, ObjectType::Conflict, Opinion::Like).is_ok());

    for _ in 0..5 {
        futures::executor::block_on(voter.do_round(0.5)).unwrap();
    }

    let mut event = None;

    let mut iter = rx.try_iter();
    while let Some(ev) = iter.next() {
        if let Event::Finalized(_) = ev {
            event = Some(ev);
        }
    }

    assert!(matches!(event, Some(Event::Finalized(_))));
}

#[tokio::test]
async fn failed_event() {
    let mock = MockOpinionGiver {
        id: random_id_string(),
        round: 0,
        round_replies: vec![Opinions::new(vec![Opinion::Dislike])], 
    };

    let opinion_giver_fn = || -> Result<Vec<Box<dyn OpinionGiver>>, Error> {
        Ok(vec![Box::new(mock.clone())])
    };

    let id = String::from("test");
    
    let mut params = FpcParameters::default();
    params.finalization_threshold = 4;
    params.cooling_off_period = 0;
    params.query_sample_size = 1;
    params.max_rounds_per_vote_context = 3;

    let (tx, rx) = flume::unbounded();

    let voter = Fpc::new(opinion_giver_fn).with_params(params).with_tx(tx);    

    assert!(voter.vote(id, ObjectType::Conflict, Opinion::Like).is_ok());

    for _ in 0..4 {
        futures::executor::block_on(voter.do_round(0.5)).unwrap();
    }

    let mut event = None;

    let mut iter = rx.try_iter();
    while let Some(ev) = iter.next() {
        if let Event::Failed(_) = ev {
            event = Some(ev);
        }
    }

    assert!(matches!(event, Some(Event::Failed(_))));
}

#[tokio::test]
async fn multiple_opinion_givers() {
    let default_params = FpcParameters::default();
    let init_opinions = vec![Opinion::Like, Opinion::Dislike];
    let expected_opinions = vec![Opinion::Like, Opinion::Dislike];
    let num_tests = 2;

    for i in 0..num_tests {
        let opinion_giver_fn = || -> Result<Vec<Box<dyn OpinionGiver>>, Error> {
            let mut opinion_givers: Vec<Box<dyn OpinionGiver>> = vec![];

            for _ in 0..default_params.query_sample_size {
                opinion_givers.push(Box::new(MockOpinionGiver {
                    id: random_id_string(),
                    round: 0,
                    round_replies: vec![Opinions::new(vec![init_opinions[i]])], 
                }));
            }

            Ok(opinion_givers)
        };

        let mut params = FpcParameters::default();
        params.finalization_threshold = 2;
        params.cooling_off_period = 2;

        let (tx, rx) = flume::unbounded();

        let voter = Fpc::new(opinion_giver_fn).with_params(params).with_tx(tx);

        assert!(voter.vote("test".to_string(), ObjectType::Conflict, init_opinions[i]).is_ok());

        let mut rounds = 0;

        let final_opinion = loop {
            assert!(voter.do_round(0.7).await.is_ok());
            rounds += 1;

            let mut iter = rx.try_iter();
            let mut final_opinion = None;

            while let Some(ev) = iter.next() {
                if let Event::Finalized(opinion_event) = ev {
                    final_opinion = Some(opinion_event.opinion);
                }
            }

            if let Some(opinion) = final_opinion {
                break opinion;
            }
        };

        assert_eq!(rounds, 5);
        assert_eq!(final_opinion, expected_opinions[i]);
    }
}
