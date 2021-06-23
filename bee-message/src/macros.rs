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