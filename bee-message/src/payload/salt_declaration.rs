// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, signature::ED25519_PUBLIC_KEY_LENGTH};

use bee_packable::Packable;

#[derive(Clone, Debug, PartialEq, Eq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Salt {
    #[packable(prefix = u32)]
    bytes: Vec<u8>,
    expiry_time: u64,
}

impl Salt {
    pub fn new(bytes: Vec<u8>, expiry_time: u64) -> Self {
        Self { 
            bytes: bytes.into(),
            expiry_time,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaltDeclarationPayload {
    version: u8,
    node_id: u32,
    salt: Salt,
    timestamp: u64,
    signature: [u8; ED25519_PUBLIC_KEY_LENGTH],
}

impl SaltDeclarationPayload {
    pub const KIND: u32 = 7;

    pub fn builder() -> SaltDeclarationPayloadBuilder {
        SaltDeclarationPayloadBuilder::new()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn signature(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] {
        &self.signature
    }
}

#[derive(Default)]
pub struct SaltDeclarationPayloadBuilder {
    version: Option<u8>,
    node_id: Option<u32>,
    salt: Option<Salt>,
    timestamp: Option<u64>,
    signature: Option<[u8; ED25519_PUBLIC_KEY_LENGTH]>,
}

impl SaltDeclarationPayloadBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_node_id(mut self, node_id: u32) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn with_salt(mut self, salt: Salt) -> Self {
        self.salt = Some(salt);
        self
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn with_signature(mut self, signature: [u8; ED25519_PUBLIC_KEY_LENGTH]) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn finish(self) -> Result<SaltDeclarationPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let node_id = self.node_id.ok_or(ValidationError::MissingField("node_id"))?;
        let salt = self.salt.ok_or(ValidationError::MissingField("salt"))?;
        let timestamp = self.timestamp.ok_or(ValidationError::MissingField("timestamp"))?;
        let signature = self.signature.ok_or(ValidationError::MissingField("signature"))?;
        
        Ok(SaltDeclarationPayload {
            version,
            node_id,
            salt,
            timestamp,
            signature,
        })
    }
}
