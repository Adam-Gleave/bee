// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! 

use crate::{
    error::Error,
    opinion::{Opinion, Opinions},
};

/// Initial "liked" value for a new `Context`.
pub const LIKED_INITIAL: f64 = -1.0;

/// Object type of a vote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectType {
    Conflict,
    Timestamp,
}

/// Builder pattern struct for instantiating a `VoteContext`.
pub struct VoteContextBuilder {
    id: String,
    object_type: ObjectType,
    opinions: Option<Opinions>,
}

impl VoteContextBuilder {
    /// Create a new `VoteContextBuilder`, defining an ID and an `ObjectType` (voting object).
    pub fn new(id: String, object_type: ObjectType) -> Self {
        Self {
            id,
            object_type,
            opinions: None,
        }
    }

    /// Set a single initial `Opinion`.
    pub fn with_initial_opinion(mut self, opinion: Opinion) -> Self {
        self.opinions = Some(Opinions::new(vec![opinion]));
        self
    }

    /// Set a list of initial `Opinion`s.
    pub fn with_initial_opinions(mut self, opinions: Opinions) -> Self {
        self.opinions = Some(opinions);
        self
    }

    /// Build a `VoteContext`.
    /// Note: this will panic if no initial opinions have been provided.
    pub fn build(self) -> Result<VoteContext, Error> {
        Ok(VoteContext {
            id: self.id,
            object_type: self.object_type,
            liked: LIKED_INITIAL,
            rounds: 0,
            opinions: self.opinions.ok_or(Error::NoInitialOpinions)?,
        })
    }
}

/// Voting context.
#[derive(Debug, Clone)]
pub struct VoteContext {
    /// Voter ID.
    id: String,
    /// Object type of the vote.
    object_type: ObjectType,
    /// The percentage of `OpinionGiver`s who liked this item on the last query.
    liked: f64,
    /// The number of voting rounds performed so far.
    rounds: u32,
    /// List of opinions formed at the end of each voting round.
    /// The first in this list is the initial opinion when this `VoteContext` was created.
    opinions: Opinions,
}

impl VoteContext {
    /// Constructs a new `VoteContext`.
    pub fn new(id: String, object_type: ObjectType, initial_opinion: Opinion) -> Self {
        Self {
            id,
            object_type,
            liked: LIKED_INITIAL,
            rounds: 0,
            opinions: Opinions::new(vec![initial_opinion]),
        }
    }

    /// Add the given `Opinion` to the `VoteContext`.
    pub fn add_opinion(&mut self, opinion: Opinion) {
        self.opinions.push(opinion);
    }

    /// Retrieve the last formed `Opinion`.
    pub fn last_opinion(&self) -> Option<Opinion> {
        self.opinions.last().copied()
    }

    /// Describes whether this `VoteContext` has been finalized.
    pub fn finalized(&self, cool_off_period: u32, finalization_threshold: u32) -> bool {
        // Check whether we have enough opinions to decide if the vote is finalised.
        if self.opinions.len() - 1 < (cool_off_period + finalization_threshold) as usize {
            false
        } else {
            // Index of the opinion that needs to be held for `finalization_threshold` number of rounds.
            let finalized_index = self.opinions.len() - finalization_threshold as usize;

            if self.opinions.len() < finalized_index + 1 {
                return false;
            }

            // Check that this opinion is held.
            if let Some(candidate_opinion) = self.opinions.get(finalized_index) {
                let subsequent_opinions = self.opinions.split_at(finalized_index + 1).1;

                for opinion in subsequent_opinions {
                    if opinion != candidate_opinion {
                        return false;
                    }
                }

                true
            } else {
                false
            }
        }
    }

    /// Describes whether the `VoteContext` is new (has not participated in a vote).
    pub fn is_new(&self) -> bool {
        self.liked == LIKED_INITIAL
    }

    /// Described whether the `VoteContext` has *just* finished its first round.
    pub fn had_first_round(&self) -> bool {
        self.rounds == 1
    }

    /// Returns the ID of the `VoteContext`
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Returns the object of the voting.
    pub fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Resturns the percentage of `OpinionGiver`s that liked the item on the last query.
    pub fn liked(&self) -> f64 {
        self.liked
    }

    /// Update the `liked` value of a `VoteContext` when new opinions are formed.
    pub fn set_liked(&mut self, liked: f64) {
        self.liked = liked;
    }

    /// Number of voting rounds completed for this item.
    pub fn rounds(&self) -> u32 {
        self.rounds
    }

    /// Indicate the completion of a voting round for this item.
    pub fn round_completed(&mut self) {
        self.rounds += 1;
    }
}
