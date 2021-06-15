// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    input::UtxoInput,
    payload::{fpc::FpcPackError, transaction::TransactionPackError, PayloadUnpackError},
};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    UnpackOptionError,
};
use crypto::Error as CryptoError;

use core::{convert::Infallible, fmt};

#[derive(Debug)]
pub enum ValidationError {
    CryptoError(CryptoError),
    DuplicateAddress(Address),
    DuplicateSignature(usize),
    DuplicateUtxo(UtxoInput),
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidAccumulatedOutput(u128),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationDataLength(usize),
    InvalidIndexationIndexLength(usize),
    InvalidInputCount(usize),
    InvalidMessageLength(usize),
    InvalidOutputCount(usize),
    InvalidParentsCount(usize),
    InvalidPayloadKind(u32),
    InvalidReferenceIndex(u16),
    InvalidSignature,
    InvalidUnlockBlockCount(usize),
    InvalidUnlockBlockReference(usize),
    MissingField(&'static str),
    ParentsNotUniqueSorted,
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
}

impl From<CryptoError> for ValidationError {
    fn from(error: CryptoError) -> Self {
        Self::CryptoError(error)
    }
}

#[derive(Debug)]
pub enum MessagePackError {
    FpcPayload(FpcPackError),
    InvalidParentsLength,
    TransactionPayload(TransactionPackError),
}

impl From<FpcPackError> for MessagePackError {
    fn from(error: FpcPackError) -> Self {
        Self::FpcPayload(error)
    }
}

impl From<TransactionPackError> for MessagePackError {
    fn from(error: TransactionPackError) -> Self {
        Self::TransactionPayload(error)
    }
}

impl From<PackPrefixError<Infallible, u32>> for MessagePackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidParentsLength,
        }
    }
}

impl From<Infallible> for MessagePackError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

#[derive(Debug)]
pub enum MessageUnpackError {
    InvalidPayloadKind(u32),
    InvalidParentsLength,
    InvalidOptionTag(u8),
    PayloadUnpackError(PayloadUnpackError),
}

impl From<UnpackPrefixError<Infallible, u32>> for MessageUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidParentsLength,
        }
    }
}

impl From<UnpackOptionError<PayloadUnpackError>> for MessageUnpackError {
    fn from(error: UnpackOptionError<PayloadUnpackError>) -> Self {
        match error {
            UnpackOptionError::Inner(e) => Self::PayloadUnpackError(e),
            UnpackOptionError::UnknownTag(tag) => Self::InvalidOptionTag(tag),
        }
    }
}

impl From<Infallible> for MessageUnpackError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

/// Error occurring when creating/parsing/validating messages.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Error {
    PackError(MessagePackError),
    UnpackError(MessageUnpackError),
    ValidationError(ValidationError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackError(_) => write!(f, "Pack error"),
            Self::UnpackError(_) => write!(f, "Unpack error."),
            Self::ValidationError(_) => write!(f, "Validation error."),
        }
    }
}

impl From<MessagePackError> for Error {
    fn from(error: MessagePackError) -> Self {
        Self::PackError(error)
    }
}

impl From<MessageUnpackError> for Error {
    fn from(error: MessageUnpackError) -> Self {
        Self::UnpackError(error)
    }
}

impl From<ValidationError> for Error {
    fn from(error: ValidationError) -> Self {
        Self::ValidationError(error)
    }
}
