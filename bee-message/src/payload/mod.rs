// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod data;
pub mod drng;
pub mod fpc;
pub mod indexation;
pub mod salt_declaration;
pub mod transaction;

use data::DataPayload;
use drng::{ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPayload};
use fpc::FpcPayload;
use indexation::IndexationPayload;
use salt_declaration::SaltDeclarationPayload;
use transaction::TransactionPayload;

use crate::Error;

use bee_packable::{Packable, Packer, UnknownTagError, UnpackError, Unpacker};

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
    /// A dRNG application message payload.
    ApplicationMessage(Box<ApplicationMessagePayload>),
    /// A dRNG beacon payload.
    Beacon(Box<BeaconPayload>),
    /// A dRNG collective beacon payload.
    CollectiveBeacon(Box<CollectiveBeaconPayload>),
    /// A pure data payload.
    Data(Box<DataPayload>),
    /// A dRNG DKG payload.
    Dkg(Box<DkgPayload>),
    /// An FPC payload.
    Fpc(Box<FpcPayload>),
    /// An indexation payload.
    Indexation(Box<IndexationPayload>),
    /// A salt declaration payload.
    SaltDeclaration(Box<SaltDeclarationPayload>),
    /// A transaction payload.
    Transaction(Box<TransactionPayload>),
}

impl Payload {
    /// Returns the payload kind of a `Payload`.
    pub fn kind(&self) -> u32 {
        match *self {
            Self::ApplicationMessage(_) => ApplicationMessagePayload::KIND,
            Self::Beacon(_) => BeaconPayload::KIND,
            Self::CollectiveBeacon(_) => CollectiveBeaconPayload::KIND,
            Self::Data(_) => DataPayload::KIND,
            Self::Dkg(_) => DkgPayload::KIND,
            Self::Fpc(_) => FpcPayload::KIND,
            Self::Indexation(_) => IndexationPayload::KIND,
            Self::SaltDeclaration(_) => SaltDeclarationPayload::KIND,
            Self::Transaction(_) => TransactionPayload::KIND,
        }
    }
}

impl From<ApplicationMessagePayload> for Payload {
    fn from(payload: ApplicationMessagePayload) -> Self {
        Self::ApplicationMessage(Box::new(payload))
    }
}

impl From<BeaconPayload> for Payload {
    fn from(payload: BeaconPayload) -> Self {
        Self::Beacon(Box::new(payload))
    }
}

impl From<CollectiveBeaconPayload> for Payload {
    fn from(payload: CollectiveBeaconPayload) -> Self {
        Self::CollectiveBeacon(Box::new(payload))
    }
}

impl From<DkgPayload> for Payload {
    fn from(payload: DkgPayload) -> Self {
        Self::Dkg(Box::new(payload))
    }
}

impl From<FpcPayload> for Payload {
    fn from(payload: FpcPayload) -> Self {
        Self::Fpc(Box::new(payload))
    }
}

impl From<IndexationPayload> for Payload {
    fn from(payload: IndexationPayload) -> Self {
        Self::Indexation(Box::new(payload))
    }
}

impl From<SaltDeclarationPayload> for Payload {
    fn from(payload: SaltDeclarationPayload) -> Self {
        Self::SaltDeclaration(Box::new(payload))
    }
}

impl From<TransactionPayload> for Payload {
    fn from(payload: TransactionPayload) -> Self {
        Self::Transaction(Box::new(payload))
    }
}

impl Packable for Payload {
    type Error = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match *self {
            Self::ApplicationMessage(ref payload) => {
                ApplicationMessagePayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::Beacon(ref payload) => {
                BeaconPayload::KIND.pack(packer)?;
                payload.pack(packer) 
            }
            Self::CollectiveBeacon(ref payload) => {
                CollectiveBeaconPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::Data(ref payload) => {
                DataPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::Dkg(ref payload) => {
                DkgPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::Fpc(ref payload) => {
                FpcPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::Indexation(ref payload) => { 
                IndexationPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::SaltDeclaration(ref payload) => {
                SaltDeclarationPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
            Self::Transaction(ref payload) => {
                TransactionPayload::KIND.pack(packer)?;
                payload.pack(packer)
            }
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        let payload = match u32::unpack(unpacker).map_err(UnpackError::coerce)? {
            ApplicationMessagePayload::KIND => Payload::ApplicationMessage(Box::new(ApplicationMessagePayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            BeaconPayload::KIND => Payload::Beacon(Box::new(BeaconPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            CollectiveBeaconPayload::KIND => Payload::CollectiveBeacon(Box::new(CollectiveBeaconPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            DataPayload::KIND => Payload::Data(Box::new(DataPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            DkgPayload::KIND => Payload::Dkg(Box::new(DkgPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            FpcPayload::KIND => Payload::Fpc(Box::new(FpcPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            IndexationPayload::KIND => Payload::Indexation(Box::new(IndexationPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            SaltDeclarationPayload::KIND => Payload::SaltDeclaration(Box::new(SaltDeclarationPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            TransactionPayload::KIND => Payload::Transaction(Box::new(TransactionPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            tag => Err(UnpackError::Packable(Self::Error::from(UnknownTagError(tag))))?,
        };

        Ok(payload)
    }

    fn packed_len(&self) -> usize {
        match *self {
            Self::ApplicationMessage(ref payload) => ApplicationMessagePayload::KIND.packed_len() + payload.packed_len(),
            Self::Beacon(ref payload) => BeaconPayload::KIND.packed_len() + payload.packed_len(),
            Self::CollectiveBeacon(ref payload) => CollectiveBeaconPayload::KIND.packed_len() + payload.packed_len(),
            Self::Data(ref payload) => DataPayload::KIND.packed_len() + payload.packed_len(),
            Self::Dkg(ref payload) => DkgPayload::KIND.packed_len() + payload.packed_len(),
            Self::Fpc(ref payload) => FpcPayload::KIND.packed_len() + payload.packed_len(),
            Self::Indexation(ref payload) => IndexationPayload::KIND.packed_len() + payload.packed_len(),
            Self::SaltDeclaration(ref payload) => SaltDeclarationPayload::KIND.packed_len() + payload.packed_len(),
            Self::Transaction(ref payload) => TransactionPayload::KIND.packed_len() + payload.packed_len(),
        }
    }
}
