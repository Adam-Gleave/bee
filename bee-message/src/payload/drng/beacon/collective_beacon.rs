// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH};
use crate::error::ValidationError;

use bee_packable::Packable;

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollectiveBeaconPayload {
    version: u8,
    instance_id: u32,
    round: u64,
    prev_signature: [u8; BEACON_SIGNATURE_LENGTH],
    signature: [u8; BEACON_SIGNATURE_LENGTH],
    distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
}

impl CollectiveBeaconPayload {
    pub const KIND: u32 = 6;

    pub fn builder() -> CollectiveBeaconPayloadBuilder {
        CollectiveBeaconPayloadBuilder::default()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
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

#[derive(Default)]
pub struct CollectiveBeaconPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    round: Option<u64>,
    prev_signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
    signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
    distributed_public_key: Option<[u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH]>,
}

impl CollectiveBeaconPayloadBuilder {
    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id = Some(instance_id);
        self
    }

    pub fn with_round(mut self, round: u64) -> Self {
        self.round = Some(round);
        self
    }

    pub fn with_prev_signature(mut self, prev_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.prev_signature = Some(prev_signature);
        self
    }

    pub fn with_signature(mut self, signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn with_distributed_public_key(
        mut self,
        distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
    ) -> Self {
        self.distributed_public_key = Some(distributed_public_key);
        self
    }

    pub fn finish(self) -> Result<CollectiveBeaconPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingField("round"))?;
        let prev_signature = self
            .prev_signature
            .ok_or(ValidationError::MissingField("prev_signature"))?;
        let signature = self.signature.ok_or(ValidationError::MissingField("signature"))?;
        let distributed_public_key = self
            .distributed_public_key
            .ok_or(ValidationError::MissingField("distributed_public_key"))?;

        Ok(CollectiveBeaconPayload {
            version,
            instance_id,
            round,
            prev_signature,
            signature,
            distributed_public_key,
        })
    }
}
