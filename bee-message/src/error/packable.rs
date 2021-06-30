// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{ValidationError, input::InputUnpackError, output::{OutputIdUnpackError, OutputUnpackError, SignatureLockedDustAllowanceUnpackError, SignatureLockedSingleUnpackError}, payload::{PayloadPackError, PayloadUnpackError, data::{DataPackError, DataUnpackError}, drng::{DkgPackError, DkgUnpackError}, fpc::{FpcPackError, FpcUnpackError}, indexation::{IndexationPackError, IndexationUnpackError}, salt_declaration::{SaltDeclarationPackError, SaltDeclarationUnpackError}, transaction::{TransactionEssencePackError, TransactionEssenceUnpackError, TransactionPackError, TransactionUnpackError}}, signature::SignatureUnlockUnpackError, unlock::{UnlockBlockUnpackError, UnlockBlocksPackError, UnlockBlocksUnpackError}};

use bee_packable::UnpackOptionError;

use core::{fmt, convert::Infallible};

#[derive(Debug)]
pub enum MessagePackError {
    Data(DataPackError),
    Dkg(DkgPackError),
    Fpc(FpcPackError),
    Indexation(IndexationPackError),
    Payload(PayloadPackError),
    SaltDeclaration(SaltDeclarationPackError),
    Transaction(TransactionPackError),
    TransactionEssence(TransactionEssencePackError),
    UnlockBlocks(UnlockBlocksPackError),
}

impl_wrapped_variant!(MessagePackError, DataPackError, MessagePackError::Data);
impl_wrapped_variant!(MessagePackError, DkgPackError, MessagePackError::Dkg);
impl_wrapped_variant!(MessagePackError, FpcPackError, MessagePackError::Fpc);
impl_wrapped_variant!(MessagePackError, IndexationPackError, MessagePackError::Indexation);
impl_wrapped_variant!(MessagePackError, PayloadPackError, MessagePackError::Payload);
impl_wrapped_variant!(MessagePackError, SaltDeclarationPackError, MessagePackError::SaltDeclaration);
impl_wrapped_variant!(MessagePackError, TransactionPackError, MessagePackError::Transaction);
impl_wrapped_variant!(MessagePackError, TransactionEssencePackError, MessagePackError::TransactionEssence);
impl_wrapped_variant!(MessagePackError, UnlockBlocksPackError, MessagePackError::UnlockBlocks);
impl_from_infallible!(MessagePackError);

impl fmt::Display for MessagePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(e) => write!(f, "Error packing Data payload: {}", e),
            Self::Dkg(e) => write!(f, "Error packing DKG payload: {}", e),
            Self::Fpc(e) => write!(f, "Error packing FPC payload: {}", e),
            Self::Indexation(e) => write!(f, "Error packing Indexation payload: {}", e),
            Self::Payload(e) => write!(f, "Error packing payload: {}", e),
            Self::SaltDeclaration(e) => write!(f, "Error packing SaltDeclaration payload: {}", e),
            Self::Transaction(e) => write!(f, "Error packing Transaction payload: {}", e),
            Self::TransactionEssence(e) => write!(f, "Error packing TransactionEssence: {}", e),
            Self::UnlockBlocks(e) => write!(f, "Error packing UnlockBlocks: {}", e), 
        }
    }
}

#[derive(Debug)]
pub enum MessageUnpackError {
    Data(DataUnpackError),
    Dkg(DkgUnpackError),
    Fpc(FpcUnpackError),
    Indexation(IndexationUnpackError),
    Input(InputUnpackError),
    InvalidPayloadKind(u32),
    InvalidOptionTag(u8),
    Output(OutputUnpackError),
    OutputId(OutputIdUnpackError),
    Payload(PayloadUnpackError),
    SaltDeclaration(SaltDeclarationUnpackError),
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceUnpackError),
    SignatureLockedSingle(SignatureLockedSingleUnpackError),
    SignatureUnlock(SignatureUnlockUnpackError),
    Transaction(TransactionUnpackError),
    TransactionEssence(TransactionEssenceUnpackError),
    UnlockBlock(UnlockBlockUnpackError),
    UnlockBlocks(UnlockBlocksUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_validated!(MessageUnpackError, IndexationUnpackError, MessageUnpackError::Indexation);
impl_wrapped_validated!(MessageUnpackError, InputUnpackError, MessageUnpackError::Input);
impl_wrapped_validated!(MessageUnpackError, OutputUnpackError, MessageUnpackError::Output);
impl_wrapped_validated!(MessageUnpackError, PayloadUnpackError, MessageUnpackError::Payload);
impl_wrapped_validated!(MessageUnpackError, TransactionUnpackError, MessageUnpackError::Transaction);
impl_wrapped_validated!(MessageUnpackError, TransactionEssenceUnpackError, MessageUnpackError::TransactionEssence);
impl_wrapped_validated!(MessageUnpackError, SignatureLockedDustAllowanceUnpackError, MessageUnpackError::SignatureLockedDustAllowance);
impl_wrapped_validated!(MessageUnpackError, SignatureLockedSingleUnpackError, MessageUnpackError::SignatureLockedSingle);
impl_wrapped_validated!(MessageUnpackError, UnlockBlockUnpackError, MessageUnpackError::UnlockBlock);
impl_wrapped_validated!(MessageUnpackError, UnlockBlocksUnpackError, MessageUnpackError::UnlockBlocks);
impl_wrapped_variant!(MessageUnpackError, DataUnpackError, MessageUnpackError::Data);
impl_wrapped_variant!(MessageUnpackError, DkgUnpackError, MessageUnpackError::Dkg);
impl_wrapped_variant!(MessageUnpackError, FpcUnpackError, MessageUnpackError::Fpc);
impl_wrapped_variant!(MessageUnpackError, SaltDeclarationUnpackError, MessageUnpackError::SaltDeclaration);
impl_wrapped_variant!(MessageUnpackError, SignatureUnlockUnpackError, MessageUnpackError::SignatureUnlock);
impl_wrapped_variant!(MessageUnpackError, ValidationError, MessageUnpackError::ValidationError);
impl_from_infallible!(MessageUnpackError);

impl From<UnpackOptionError<MessageUnpackError>> for MessageUnpackError {
    fn from(error: UnpackOptionError<MessageUnpackError>) -> Self {
        match error {
            UnpackOptionError::Inner(error) => error,
            UnpackOptionError::UnknownTag(tag) => Self::InvalidOptionTag(tag),
        }
    }
}

impl fmt::Display for MessageUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(e) => write!(f, "Error unpacking Data payload: {}", e),
            Self::Dkg(e) => write!(f, "Error unpacking DKG payload: {}", e),
            Self::Fpc(e) => write!(f, "Error unpacking FPC payload: {}", e),
            Self::Indexation(e) => write!(f, "Error unpacking Indexation payload: {}", e),
            Self::Input(e) => write!(f, "Error unpacking Input: {}", e),
            Self::InvalidPayloadKind(kind) => write!(f, "Invalid payload kind: {}.", kind),
            Self::InvalidOptionTag(tag) => write!(f, "Invalid tag for Option: {} is not 0 or 1", tag),
            Self::Output(e) => write!(f, "Error unpacking Output: {}", e),
            Self::OutputId(e) => write!(f, "Error unpacking OutputId: {}", e),
            Self::Payload(e) => write!(f, "Error unpacking payload: {}", e),
            Self::SaltDeclaration(e) => write!(f, "Error unpacking SaltDeclaration payload: {}", e),
            Self::SignatureLockedDustAllowance(e) => write!(f, "Error unpacking SignatureLockedDustAllowance: {}", e),
            Self::SignatureLockedSingle(e) => write!(f, "Error unpacking SignatureLockedSingle: {}", e),
            Self::SignatureUnlock(e) => write!(f, "Error unpacking SignatureUnlock: {}", e),
            Self::Transaction(e) => write!(f, "Error unpacking Transaction payload: {}", e),
            Self::TransactionEssence(e) => write!(f, "Error unpacking TransactionEssence: {}", e),
            Self::UnlockBlock(e) => write!(f, "Error unpacking UnlockBlock: {}", e),
            Self::UnlockBlocks(e) => write!(f, "Error unpacking UnlockBlocks: {}", e),
            Self::ValidationError(e) => write!(f, "Validation error occured while unpacking: {}", e),
        }
    }
}
