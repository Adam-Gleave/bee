// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    input::UtxoInput,
    payload::{PayloadPackError, PayloadUnpackError},
};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    UnpackOptionError,
};
use crypto::Error as CryptoError;

use alloc::string::String;
use core::{convert::Infallible, fmt};

#[derive(Debug)]
pub enum ValidationError {
    CryptoError(CryptoError),
    DuplicateAddress(Address),
    DuplicateSignature(usize),
    DuplicateUtxo(UtxoInput),
    InputUnlockBlockCountMismatch(usize, usize),
    InvalidAccumulatedOutput(u128),
    InvalidAddress,
    InvalidAmount(u64),
    InvalidDustAllowanceAmount(u64),
    InvalidHexadecimalChar(String),
    InvalidHexadecimalLength(usize, usize),
    InvalidIndexationDataLength(usize),
    InvalidIndexationIndexLength(usize),
    InvalidInputCount(usize),
    InvalidMessageLength(usize),
    InvalidOutputCount(usize),
    InvalidOutputIndex(u16),
    InvalidParentsCount(usize),
    InvalidPayloadKind(u32),
    InvalidReferenceIndex(u16),
    InvalidSignature,
    InvalidUnlockBlockCount(usize),
    InvalidUnlockBlockReference(usize),
    MissingField(&'static str),
    ParentsNotUniqueSorted,
    SignaturePublicKeyMismatch(String, String),
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CryptoError(e) => write!(f, "Cryptographic error: {}.", e),
            Self::DuplicateAddress(address) => write!(f, "Duplicate address {:?} in outputs.", address),
            Self::DuplicateSignature(index) => {
                write!(f, "Duplicate signature at index: {}.", index)
            }
            Self::DuplicateUtxo(utxo) => write!(f, "Duplicate UTX {:?} in inputs.", utxo),
            Self::InputUnlockBlockCountMismatch(input, block) => {
                write!(
                    f,
                    "Input countr and unlock block count mismatch: {} != {}.",
                    input, block,
                )
            }
            Self::InvalidAccumulatedOutput(value) => write!(f, "Invalid accumulated output balance: {}.", value),
            Self::InvalidAddress => write!(f, "Invalid address provided."),
            Self::InvalidAmount(amount) => write!(f, "Invalid amount: {}.", amount),
            Self::InvalidDustAllowanceAmount(amount) => write!(f, "Invalid dust allowance amount: {}.", amount),
            Self::InvalidHexadecimalChar(hex) => write!(f, "Invalid hexadecimal character: {}.", hex),
            Self::InvalidHexadecimalLength(expected, actual) =>  {
                write!(f, "Invalid hexadecimal length: expected {} got {}.", expected, actual)
            }
            Self::InvalidIndexationDataLength(len) => {
                write!(f, "Invalid indexation data length: {}.", len)
            }
            Self::InvalidIndexationIndexLength(len) => {
                write!(f, "Invalid indexation index length: {}.", len)
            }
            Self::InvalidInputCount(count) => write!(f, "Invalid input count: {}.", count),
            Self::InvalidMessageLength(len) => write!(f, "Invalid message length: {}.", len),
            Self::InvalidOutputCount(count) => write!(f, "Invalid output count: {}.", count),
            Self::InvalidOutputIndex(index) => write!(f, "Inavlid output index: {}.", index),
            Self::InvalidParentsCount(count) => write!(f, "Invalid parents count: {}.", count),
            Self::InvalidPayloadKind(kind) => write!(f, "Invalid payload kind: {}.", kind),
            Self::InvalidReferenceIndex(index) => write!(f, "Invalid reference index: {}.", index),
            Self::InvalidSignature => write!(f, "Invalid signature provided."),
            Self::InvalidUnlockBlockCount(count) => write!(f, "Invalid unlock block count: {}.", count),
            Self::InvalidUnlockBlockReference(index) => {
                write!(f, "Invalid unlock block reference: {}", index)
            }
            Self::MissingField(field) => write!(f, "Missing required field: {}.", field),
            Self::ParentsNotUniqueSorted => write!(f, "Parents not unique and/or sorted."),
            Self::SignaturePublicKeyMismatch(expected, actual) => {
                write!(
                    f,
                    "Signature public key mismatch: expected {}, got {}.",
                    expected, actual,
                )
            }
            Self::TransactionInputsNotSorted => {
                write!(f, "Transaction inputs are not sorted.")
            }
            Self::TransactionOutputsNotSorted => {
                write!(f, "Transaction outputs are not sorted.")
            }
        }
    }
}

impl From<CryptoError> for ValidationError {
    fn from(error: CryptoError) -> Self {
        Self::CryptoError(error)
    }
}

#[derive(Debug)]
pub enum MessagePackError {
    InvalidParentsLength,
    PayloadPackError(PayloadPackError),
}

impl fmt::Display for MessagePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParentsLength => write!(f, "Invalid parents vector length."),
            Self::PayloadPackError(e) => write!(f, "{}", e),
        }
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

impl From<PayloadPackError> for MessagePackError {
    fn from(error: PayloadPackError) -> Self {
        Self::PayloadPackError(error)
    }
}

#[derive(Debug)]
pub enum MessageUnpackError {
    InvalidPayloadKind(u32),
    InvalidParentsLength,
    InvalidOptionTag(u8),
    PayloadUnpackError(PayloadUnpackError),
}

impl fmt::Display for MessageUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayloadKind(kind) => write!(f, "Invalid payload kind: {}.", kind),
            Self::InvalidParentsLength => write!(f, "Invalid parents vector length."),
            Self::InvalidOptionTag(tag) => write!(f, "Invalid tag for Option: {} is not 0 or 1.", tag),
            Self::PayloadUnpackError(e) => write!(f, "{}", e),
        }
    }
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
            Self::PackError(e) => write!(f, "{}", e),
            Self::UnpackError(e) => write!(f, "{}", e),
            Self::ValidationError(e) => write!(f, "Validation error: {}", e),
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
