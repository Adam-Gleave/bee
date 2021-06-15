// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::Packable;

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPayload {
    version: u8,
    #[packable(prefix = u32)]
    data: Vec<u8>,
}

impl DataPayload {
    pub const KIND: u32 = 0;

    pub fn new(version: u8, data: Vec<u8>) -> Self {
        Self {
            version,
            data: data.into(),
        }
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}
