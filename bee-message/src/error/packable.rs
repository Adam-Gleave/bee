// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::{PayloadPackError, PayloadUnpackError}, ValidationError};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    UnpackOptionError,
};

use core::{convert::Infallible, fmt};

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
    ValidationError(ValidationError),
}

impl MessageUnpackError {
    fn validation_error(&self) -> Option<&ValidationError> {
        match self {
            Self::ValidationError(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for MessageUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayloadKind(kind) => write!(f, "Invalid payload kind: {}.", kind),
            Self::InvalidParentsLength => write!(f, "Invalid parents vector length."),
            Self::InvalidOptionTag(tag) => write!(f, "Invalid tag for Option: {} is not 0 or 1.", tag),
            Self::PayloadUnpackError(e) => write!(f, "{}", e),
	        Self::ValidationError(e) => write!(f, "{}", e),
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

impl From<ValidationError> for MessageUnpackError {
    fn from(error: ValidationError) -> Self {
        Self::ValidationError(error)
    }
}

impl From<Infallible> for MessageUnpackError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}
