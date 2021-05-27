// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, MessageId};

// use bee_common::packable::{Packable, Read, Write};

use core::ops::Deref;

/// Provides a convenient collection of `Timestamp`s.
/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamps(Vec<Timestamp>);

impl Deref for Timestamps {
    type Target = [Timestamp];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl Packable for Timestamps {
//     type Error = Error;

//     fn packed_len(&self) -> usize {
//         let timestamps_len = self.len() * self.first().map_or_else(|| 0, |item| item.packed_len());

//         0u32.packed_len() + timestamps_len 
//     }

//     fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
//         (self.len() as u32).pack(writer)?;

//         for timestamp in self.iter() {
//             timestamp.pack(writer)?;
//         }

//         Ok(())
//     }

//     fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
//         let timestamps_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;

//         let mut inner = Vec::with_capacity(timestamps_len);
//         for _ in 0..timestamps_len {
//             inner.push(Timestamp::unpack_inner::<R, CHECK>(reader)?);
//         }

//         Ok(Self(inner))
//     }
// }

/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamp {
    /// ID of the message that contains the timestamp.
    message_id: MessageId,
    /// The nodes opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl Timestamp {
    /// Returns the ID of message that contains the timestamp. 
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
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

// impl Packable for Timestamp {
//     type Error = Error;

//     fn packed_len(&self) -> usize {
//         self.message_id.packed_len()
//             + self.opinion.packed_len()
//             + self.round.packed_len()
//     }

//     fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
//         self.message_id.pack(writer)?;
//         self.opinion.pack(writer)?;
//         self.round.pack(writer)?;

//         Ok(())
//     }

//     fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
//         let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
//         let opinion = u8::unpack_inner::<R, CHECK>(reader)?;
//         let round = u8::unpack_inner::<R, CHECK>(reader)?;

//         Ok(Self {
//             message_id,
//             opinion,
//             round,
//         })
//     }
// }
