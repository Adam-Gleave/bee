// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! impl_wrapped_variant {
    ($dst:ty, $src:ty, $variant:path) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> $dst {
                $variant(src)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_wrapped_validated {
    ($dst:ident, $src:ident, $variant:path) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> $dst {
                match src {
                    $src::ValidationError(error) => $dst::ValidationError(error),
                    error => $variant(error),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_infallible {
    ($type:ty) => {
        impl From<core::convert::Infallible> for $type {
            fn from(i: Infallible) -> $type {
                match i {}
            }
        }
    };
}
