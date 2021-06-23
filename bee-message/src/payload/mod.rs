// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod data;
pub mod drng;
pub mod fpc;
pub mod indexation;
pub mod salt_declaration;
pub mod transaction;

use crate::ValidationError;

use data::{DataPayload, DataPackError, DataUnpackError};
use drng::{ApplicationMessagePayload, DkgPayload, DkgPackError, DkgUnpackError, BeaconPayload, CollectiveBeaconPayload};
use fpc::{FpcPayload, FpcPackError, FpcUnpackError};
use indexation::{IndexationPayload, IndexationPackError, IndexationUnpackError};
use salt_declaration::{SaltDeclarationPayload, SaltDeclarationPackError, SaltDeclarationUnpackError};
use transaction::{TransactionPayload, TransactionPackError, TransactionUnpackError};

use bee_packable::{PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker};

use alloc::boxed::Box;
use core::{fmt, convert::Infallible};

#[derive(Debug)]
pub enum PayloadPackError {
    Data(DataPackError),
    Dkg(DkgPackError),
    Fpc(FpcPackError),
    Indexation(IndexationPackError),
    SaltDeclaration(SaltDeclarationPackError),
    Transaction(TransactionPackError),
}

impl_wrapped_variant!(PayloadPackError, DataPackError, PayloadPackError::Data);
impl_wrapped_variant!(PayloadPackError, DkgPackError, PayloadPackError::Dkg);
impl_wrapped_variant!(PayloadPackError, FpcPackError, PayloadPackError::Fpc);
impl_wrapped_variant!(PayloadPackError, IndexationPackError, PayloadPackError::Indexation);
impl_wrapped_variant!(PayloadPackError, SaltDeclarationPackError, PayloadPackError::SaltDeclaration);
impl_wrapped_variant!(PayloadPackError, TransactionPackError, PayloadPackError::Transaction);
impl_from_infallible!(PayloadPackError);

impl fmt::Display for PayloadPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(e) => write!(f, "Error packing data payload: {}.", e),
            Self::Dkg(e) => write!(f, "Error packing DKG payload: {}", e),
            Self::Fpc(e) => write!(f, "Error packing FPC payload: {}.", e),
            Self::Indexation(e) => write!(f, "Error packing indexation payload: {}", e),
            Self::SaltDeclaration(e) => write!(f, "Error packing salt declaration payload: {}", e),
            Self::Transaction(e) => write!(f, "Error packing transaction payload: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum PayloadUnpackError {
    Data(DataUnpackError),
    Dkg(DkgUnpackError),
    Fpc(FpcUnpackError),
    Indexation(IndexationUnpackError),
    InvalidPayloadKind(u32),
    SaltDeclaration(SaltDeclarationUnpackError),
    Transaction(TransactionUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(PayloadUnpackError, DataUnpackError, PayloadUnpackError::Data);
impl_wrapped_variant!(PayloadUnpackError, DkgUnpackError, PayloadUnpackError::Dkg);
impl_wrapped_variant!(PayloadUnpackError, FpcUnpackError, PayloadUnpackError::Fpc);
impl_wrapped_variant!(PayloadUnpackError, IndexationUnpackError, PayloadUnpackError::Indexation);
impl_wrapped_variant!(PayloadUnpackError, SaltDeclarationUnpackError, PayloadUnpackError::SaltDeclaration);
impl_wrapped_variant!(PayloadUnpackError, ValidationError, PayloadUnpackError::ValidationError);
impl_from_infallible!(PayloadUnpackError);

impl fmt::Display for PayloadUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(e) => write!(f, "Error unpacking data payload: {}", e),
            Self::Dkg(e) => write!(f, "Error unpacking DKG payload: {}", e),
            Self::Fpc(e) => write!(f, "Error unpacking FPC payload: {}.", e),
            Self::Indexation(e) => write!(f, "Error unpacking indexation payload: {}.", e),
            Self::InvalidPayloadKind(kind) => write!(f, "Invalid payload kind: {}.", kind),
            Self::SaltDeclaration(e) => write!(f, "Error unpacking salt declaration payload: {}", e),
            Self::Transaction(e) => write!(f, "Error unpacking transaction payload: {}", e),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

impl From<TransactionUnpackError> for PayloadUnpackError {
    fn from(error: TransactionUnpackError) -> Self {
        match error {
            TransactionUnpackError::ValidationError(error) => Self::ValidationError(error),
            error => Self::Transaction(error),
        }
    }
}

impl From<UnknownTagError<u32>> for PayloadUnpackError {
    fn from(error: UnknownTagError<u32>) -> Self {
        match error {
            UnknownTagError(tag) => Self::InvalidPayloadKind(tag),
        }
    }
}

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
    type PackError = PayloadPackError;
    type UnpackError = PayloadUnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        match *self {
            Self::ApplicationMessage(ref payload) => {
                ApplicationMessagePayload::KIND
                    .pack(packer)
                    .map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::Beacon(ref payload) => {
                BeaconPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::CollectiveBeacon(ref payload) => {
                CollectiveBeaconPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::Data(ref payload) => {
                DataPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::Dkg(ref payload) => {
                DkgPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::Fpc(ref payload) => {
                FpcPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::Indexation(ref payload) => {
                IndexationPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::SaltDeclaration(ref payload) => {
                SaltDeclarationPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
            Self::Transaction(ref payload) => {
                TransactionPayload::KIND.pack(packer).map_err(PackError::coerce)?;
                payload.pack(packer).map_err(PackError::coerce)
            }
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let payload = match u32::unpack(unpacker).map_err(UnpackError::coerce)? {
            ApplicationMessagePayload::KIND => Payload::ApplicationMessage(Box::new(
                ApplicationMessagePayload::unpack(unpacker).map_err(UnpackError::coerce)?,
            )),
            BeaconPayload::KIND => {
                Payload::Beacon(Box::new(BeaconPayload::unpack(unpacker).map_err(UnpackError::coerce)?))
            }
            CollectiveBeaconPayload::KIND => Payload::CollectiveBeacon(Box::new(
                CollectiveBeaconPayload::unpack(unpacker).map_err(UnpackError::coerce)?,
            )),
            DataPayload::KIND => Payload::Data(Box::new(DataPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            DkgPayload::KIND => Payload::Dkg(Box::new(DkgPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            FpcPayload::KIND => Payload::Fpc(Box::new(FpcPayload::unpack(unpacker).map_err(UnpackError::coerce)?)),
            IndexationPayload::KIND => Payload::Indexation(Box::new(
                IndexationPayload::unpack(unpacker).map_err(UnpackError::coerce)?,
            )),
            SaltDeclarationPayload::KIND => Payload::SaltDeclaration(Box::new(
                SaltDeclarationPayload::unpack(unpacker).map_err(UnpackError::coerce)?,
            )),
            TransactionPayload::KIND => Payload::Transaction(Box::new(
                TransactionPayload::unpack(unpacker).map_err(UnpackError::coerce)?,
            )),
            tag => Err(UnpackError::Packable(Self::UnpackError::from(UnknownTagError(tag))))?,
        };

        Ok(payload)
    }

    fn packed_len(&self) -> usize {
        match *self {
            Self::ApplicationMessage(ref payload) => {
                ApplicationMessagePayload::KIND.packed_len() + payload.packed_len()
            }
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
