// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0
use std::{collections::HashMap, fmt, ops::Deref};

pub trait OpinionGiver {
    // TODO
}

/// Represents `Opinion`s queried from an `OpinionGiver`.
pub struct QueriedOpinions {
    /// ID of the `OpinionGiver`.
    opinion_giver_id: String,
    /// Map of IDs to `Opinion`s.
    opinions: HashMap<String, Opinion>,
    /// The amount of times the `OpinionGiver`'s opinion has been counted.
    /// Usually this number is 1, but due to randomisation of the queried `OpinionGiver`s,
    /// the same `OpinionGiver`'s opinions might be counted multiple times.
    times_counted: u32,
}

/// Defines an opinion.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opinion {
    /// Defines a "like" opinion.
    Like    = 0x01,
    /// Defines a "dislike" opinion.
    Dislike = 0x02,
    /// Defines an "unknown" opinion.
    Unknown = 0x04,
}

impl fmt::Display for Opinion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Wrapper tuple struct for a collection of opinions.
pub struct Opinions(Vec<Opinion>);

impl Deref for Opinions {
    type Target = Vec<Opinion>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Opinions {
    pub fn new(inner: Vec<Opinion>) -> Self {
        Self(inner)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Opinion> + '_ {
        self.0.iter()
    }
}
