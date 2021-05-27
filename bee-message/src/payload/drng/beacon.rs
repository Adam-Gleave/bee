// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

pub const BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH: usize = 48;
pub const BEACON_PARTIAL_PUBLIC_KEY_LENGTH: usize = 96;
pub const BEACON_SIGNATURE_LENGTH: usize = 96;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BeaconSubpayload {
    Beacon(RegularBeaconSubpayload),
    CollectiveBeacon(CollectiveBeaconSubpayload),
}

impl BeaconSubpayload {
    pub fn kind(&self) -> u8 {
        // TODO verify values with goshimmer.
        match *self {
            Self::Beacon(_) => 0,
            Self::CollectiveBeacon(_) => 1,
        }
    }
}

impl From<RegularBeaconSubpayload> for BeaconSubpayload {
    fn from(subpayload: RegularBeaconSubpayload) -> Self {
        Self::Beacon(subpayload)
    }
}

impl From<CollectiveBeaconSubpayload> for BeaconSubpayload {
    fn from(subpayload: CollectiveBeaconSubpayload) -> Self {
        Self::CollectiveBeacon(subpayload)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegularBeaconSubpayload {
    round: u64,
    partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH],
    partial_signature: [u8; BEACON_SIGNATURE_LENGTH],
}

impl RegularBeaconSubpayload {
    pub fn new(
        round: u64, 
        partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH], 
        partial_signature: [u8; BEACON_SIGNATURE_LENGTH],
    ) -> Self {
        Self { round, partial_public_key, partial_signature }
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn partial_public_key(&self) -> &[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH] {
        &self.partial_public_key
    }

    pub fn partial_signature(&self) -> &[u8; BEACON_SIGNATURE_LENGTH] {
        &self.partial_signature
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollectiveBeaconSubpayload {
    round: u64,
    prev_signature: [u8; BEACON_SIGNATURE_LENGTH],
    signature: [u8; BEACON_SIGNATURE_LENGTH],
    distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
}

impl CollectiveBeaconSubpayload {
    pub fn new(
        round: u64,
        prev_signature: [u8; BEACON_SIGNATURE_LENGTH],
        signature: [u8; BEACON_SIGNATURE_LENGTH],
        distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
    ) -> Self {
        Self { round, prev_signature, signature, distributed_public_key }
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn prev_signature(&self) -> &[u8; BEACON_SIGNATURE_LENGTH] {
        &self.prev_signature
    }

    pub fn signature(&self) -> &[u8; BEACON_SIGNATURE_LENGTH] {
        &self.signature
    }

    pub fn distributed_public_key(&self) -> &[u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH] {
        &self.distributed_public_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconPayload {
    version: u8,
    instance_id: u32,
    subpayload: BeaconSubpayload,
}

impl BeaconPayload {
    // TODO verify values with goshimmer.
    pub const KIND: u32 = 4;

    pub fn builder() -> BeaconPayloadBuilder {
        BeaconPayloadBuilder::new()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    pub fn subpayload(&self) -> &BeaconSubpayload {
        &self.subpayload
    }
}

pub struct BeaconPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    subpayload: Option<BeaconSubpayload>,
}

impl BeaconPayloadBuilder {
    pub fn new() -> Self {
        Self {
            version: None,
            instance_id: None,
            subpayload: None,
        }
    }

    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id = Some(instance_id);
        self
    }

    pub fn with_subpayload(mut self, subpayload: BeaconSubpayload) -> Self {
        self.subpayload = Some(subpayload);
        self
    }

    pub fn finish(self) -> Result<BeaconPayload, Error> {
        let version = self.version.ok_or(Error::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(Error::MissingField("instance_id"))?;
        let subpayload = self.subpayload.ok_or(Error::MissingField("subpayload"))?;

        Ok(BeaconPayload {
            version,
            instance_id,
            subpayload,
        })
    }
}