// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod packable;
mod validation;

pub use packable::{MessagePackError, MessageUnpackError};
pub use validation::ValidationError;

use core::fmt;

#[derive(Debug)]
pub enum Error {
    PackError(MessagePackError),
    UnpackError(MessageUnpackError),
    ValidationError(ValidationError),
}

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