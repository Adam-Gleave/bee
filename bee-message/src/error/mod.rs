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

impl_wrapped_variant!(Error, MessagePackError, Error::PackError);
impl_wrapped_variant!(Error, MessageUnpackError, Error::UnpackError);
impl_wrapped_variant!(Error, ValidationError, Error::ValidationError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackError(e) => write!(f, "{}", e),
            Self::UnpackError(e) => write!(f, "{}", e),
            Self::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}
