// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{BEACON_PARTIAL_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH};
use crate::error::ValidationError;

use bee_packable::Packable;
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconPayload {
    version: u8,
    instance_id: u32,
    round: u64,
    partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH],
    partial_signature: [u8; BEACON_SIGNATURE_LENGTH],
}

impl BeaconPayload {
    pub const KIND: u32 = 5;

    pub fn builder() -> BeaconPayloadBuilder {
        BeaconPayloadBuilder::new()
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

    pub fn partial_public_key(&self) -> &[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH] {
        &self.partial_public_key
    }

    pub fn partial_signature(&self) -> &[u8; BEACON_SIGNATURE_LENGTH] {
        &self.partial_signature
    }
}

#[derive(Default)]
pub struct BeaconPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    round: Option<u64>,
    partial_public_key: Option<[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]>,
    partial_signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
}

impl BeaconPayloadBuilder {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn with_partial_public_key(mut self, partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]) -> Self {
        self.partial_public_key = Some(partial_public_key);
        self
    }

    pub fn with_partial_signature(mut self, partial_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.partial_signature = Some(partial_signature);
        self
    }

    pub fn finish(self) -> Result<BeaconPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingField("round"))?;
        let partial_public_key = self
            .partial_public_key
            .ok_or(ValidationError::MissingField("partial_public_key"))?;
        let partial_signature = self
            .partial_signature
            .ok_or(ValidationError::MissingField("partial_signature"))?;

        Ok(BeaconPayload {
            version,
            instance_id,
            round,
            partial_public_key,
            partial_signature,
        })
    }
}
