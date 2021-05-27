// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::transaction::TransactionId, Error};

// use bee_common::packable::{Packable, Read, Write};

use core::ops::Deref;

/// Provides a convenient collection of `Conflict`s.
/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Conflicts(Vec<Conflict>);

impl Deref for Conflicts {
    type Target = [Conflict];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl Packable for Conflicts {
//     type Error = Error;

//     fn packed_len(&self) -> usize {
//         let conflicts_len = self.len() * self.first().map_or_else(|| 0, |item| item.packed_len());

//         0u32.packed_len() + conflicts_len 
//     }

//     fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
//         (self.len() as u32).pack(writer)?;

//         for conflict in self.iter() {
//             conflict.pack(writer)?;
//         }

//         Ok(())
//     }

//     fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
//         let conflicts_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;

//         let mut inner = Vec::with_capacity(conflicts_len);
//         for _ in 0..conflicts_len {
//             inner.push(Conflict::unpack_inner::<R, CHECK>(reader)?);
//         }

//         Ok(Self(inner))
//     }
// }

/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Conflict {
    /// ID of the conflicting transaction.
    transaction_id: TransactionId,
    /// The nodes opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl Conflict {
    /// Returns the ID of the conflicting transaction. 
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the nodes opinion value in a given round.
    pub fn opinion(&self) -> u8 {
        self.opinion
    }

    /// Returns the voting round number.
    pub fn round(&self) -> u8 {
        self.round
    }
}

// impl Packable for Conflict {
//     type Error = Error;

//     fn packed_len(&self) -> usize {
//         self.transaction_id.packed_len()
//             + self.opinion.packed_len()
//             + self.round.packed_len()
//     }

//     fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
//         self.transaction_id.pack(writer)?;
//         self.opinion.pack(writer)?;
//         self.round.pack(writer)?;

//         Ok(())
//     }

//     fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
//         let transaction_id = TransactionId::unpack_inner::<R, CHECK>(reader)?;
//         let opinion = u8::unpack_inner::<R, CHECK>(reader)?;
//         let round = u8::unpack_inner::<R, CHECK>(reader)?;

//         Ok(Self {
//             transaction_id,
//             opinion,
//             round,
//         })
//     }
// }
