// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationMessagePayload {
    version: u8,
    instance_id: u32,
}

impl ApplicationMessagePayload {
    // TODO verify values with goshimmer.
    pub const KIND: u32 = 6;

    pub fn new(version: u8, instance_id: u32) -> Self {
        Self { version, instance_id } 
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }
}