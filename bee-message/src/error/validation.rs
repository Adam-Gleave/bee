// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    input::UtxoInput,
};

use crypto::Error as CryptoError;

use alloc::string::String;
use core::fmt;

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
    InvalidStrongParentsCount(usize),
    InvalidUnlockBlockCount(usize),
    InvalidUnlockBlockReference(usize),
    MissingField(&'static str),
    ParentsNotUniqueSorted,
    SignaturePublicKeyMismatch(String, String),
    TransactionInputsNotSorted,
    TransactionOutputsNotSorted,
}

impl_wrapped_variant!(ValidationError, CryptoError, ValidationError::CryptoError);

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
            Self::InvalidStrongParentsCount(count) => write!(f, "Invalid strong parents count: {}", count),
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
