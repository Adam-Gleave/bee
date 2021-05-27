// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod drng;
pub mod fpc;
pub mod indexation;
pub mod transaction;

// use drng::{ApplicationMessagePayload, BeaconPayload, DkgPayload};
use fpc::FpcPayload;
use indexation::IndexationPayload;
use transaction::TransactionPayload;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use alloc::boxed::Box;

/// A generic payload that can represent different types defining message payloads.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Payload {
    // /// A dRNG application message payload.
    // ApplicationMessage(Box<ApplicationMessagePayload>),
    // /// A dRNG beacon payload.
    // Beacon(Box<BeaconPayload>),
    // /// A dRNG DKG payload.
    // Dkg(Box<DkgPayload>),
    /// A transaction payload.
    Transaction(Box<TransactionPayload>),
    /// An indexation payload.
    Indexation(Box<IndexationPayload>),
    /// An FPC payload.
    Fpc(Box<FpcPayload>),
}

impl Payload {
    /// Returns the payload kind of a `Payload`.
    pub fn kind(&self) -> u32 {
        match *self {
            // Self::ApplicationMessage(_) => ApplicationMessagePayload::KIND,
            // Self::Beacon(_) => BeaconPayload::KIND,
            // Self::Dkg(_) => DkgPayload::KIND,
            Self::Transaction(_) => TransactionPayload::KIND,
            Self::Indexation(_) => IndexationPayload::KIND,
            Self::Fpc(_) => FpcPayload::KIND,
        }
    }
}

// impl From<ApplicationMessagePayload> for Payload {
//     fn from(payload: ApplicationMessagePayload) -> Self {
//         Self::ApplicationMessage(Box::new(payload))
//     }
// }

// impl From<BeaconPayload> for Payload {
//     fn from(payload: BeaconPayload) -> Self {
//         Self::Beacon(Box::new(payload))
//     }
// }

// impl From<DkgPayload> for Payload {
//     fn from(payload: DkgPayload) -> Self {
//         Self::Dkg(Box::new(payload))
//     }
// }

impl From<TransactionPayload> for Payload {
    fn from(payload: TransactionPayload) -> Self {
        Self::Transaction(Box::new(payload))
    }
}

impl From<IndexationPayload> for Payload {
    fn from(payload: IndexationPayload) -> Self {
        Self::Indexation(Box::new(payload))
    }
}

impl From<FpcPayload> for Payload {
    fn from(payload: FpcPayload) -> Self {
        Self::Fpc(Box::new(payload))
    }
}

impl Packable for Payload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match *self {
            Self::Transaction(ref payload) => TransactionPayload::KIND.packed_len() + payload.packed_len(),
            Self::Indexation(ref payload) => IndexationPayload::KIND.packed_len() + payload.packed_len(),
            Self::Fpc(ref payload) => FpcPayload::KIND.packed_len() + payload.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Transaction(payload) => {
                TransactionPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Indexation(payload) => {
                IndexationPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Fpc(payload) => {
                FpcPayload::KIND.pack(writer)?;
                payload.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u32::unpack_inner::<R, CHECK>(reader)? {
            TransactionPayload::KIND => TransactionPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            IndexationPayload::KIND => IndexationPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            FpcPayload::KIND => FpcPayload::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidPayloadKind(k)),
        })
    }
}

/// Returns the packed length of an optional payload.
pub fn option_payload_packed_len(payload: Option<&Payload>) -> usize {
    0u32.packed_len() + payload.map_or(0, Packable::packed_len)
}

/// Packs an optional payload to a writer.
pub fn option_payload_pack<W: Write>(writer: &mut W, payload: Option<&Payload>) -> Result<(), Error> {
    if let Some(payload) = payload {
        (payload.packed_len() as u32).pack(writer)?;
        payload.pack(writer)?;
    } else {
        0u32.pack(writer)?;
    }

    Ok(())
}

/// Unpacks an optional payload from a reader.
pub fn option_payload_unpack<R: Read + ?Sized, const CHECK: bool>(
    reader: &mut R,
) -> Result<(usize, Option<Payload>), Error> {
    let payload_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;

    if payload_len > 0 {
        let payload = Payload::unpack_inner::<R, CHECK>(reader)?;
        if payload_len != payload.packed_len() {
            Err(Error::InvalidPayloadLength(payload_len, payload.packed_len()))
        } else {
            Ok((payload_len, Some(payload)))
        }
    } else {
        Ok((0, None))
    }
}
