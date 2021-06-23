// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{ValidationError, parents::{ParentsPackError, ParentsUnpackError}, payload::{PayloadPackError, PayloadUnpackError}};

use bee_packable::{UnpackOptionError};

use core::{convert::Infallible, fmt};

#[derive(Debug)]
pub enum MessagePackError {
    ParentsPackError(ParentsPackError),
    PayloadPackError(PayloadPackError),
}

impl_wrapped_variant!(MessagePackError, ParentsPackError, MessagePackError::ParentsPackError);
impl_wrapped_variant!(MessagePackError, PayloadPackError, MessagePackError::PayloadPackError);

impl fmt::Display for MessagePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParentsPackError(e) => write!(f, "{}", e),
            Self::PayloadPackError(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub enum MessageUnpackError {
    InvalidPayloadKind(u32),
    InvalidOptionTag(u8),
    ParentsUnpackError(ParentsUnpackError),
    PayloadUnpackError(PayloadUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(MessageUnpackError, ValidationError, MessageUnpackError::ValidationError);

impl MessageUnpackError {
    fn validation_error(&self) -> Option<&ValidationError> {
        match self {
            Self::ValidationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<UnpackOptionError<PayloadUnpackError>> for MessageUnpackError {
    fn from(error: UnpackOptionError<PayloadUnpackError>) -> Self {
        match error {
            UnpackOptionError::Inner(error) => match error {
                PayloadUnpackError::ValidationError(error) => Self::ValidationError(error),
                error => Self::PayloadUnpackError(error),
            }
            UnpackOptionError::UnknownTag(tag) => Self::InvalidOptionTag(tag),
        }
    }
}

impl From<ParentsUnpackError> for MessageUnpackError {
    fn from(error: ParentsUnpackError) -> Self {
        match error {
            ParentsUnpackError::ValidationError(error) => Self::ValidationError(error),
            error => Self::ParentsUnpackError(error),
        }
    }
}

impl From<Infallible> for MessageUnpackError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

impl fmt::Display for MessageUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayloadKind(kind) => write!(f, "Invalid payload kind: {}.", kind),
            Self::InvalidOptionTag(tag) => write!(f, "Invalid tag for Option: {} is not 0 or 1.", tag),
            Self::ParentsUnpackError(e) => write!(f, "{}", e),
            Self::PayloadUnpackError(e) => write!(f, "{}", e),
	        Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

