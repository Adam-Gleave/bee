// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{BEACON_PARTIAL_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH};
use crate::ValidationError;

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use alloc::boxed::Box;
use core::convert::{Infallible, TryInto};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconPayload {
    version: u8,
    instance_id: u32,
    round: u64,
    partial_public_key: Box<[u8]>,
    partial_signature: Box<[u8]>,
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

    pub fn partial_public_key(&self) -> &[u8] {
        &self.partial_public_key
    }

    pub fn partial_signature(&self) -> &[u8] {
        &self.partial_signature
    }
}

impl Packable for BeaconPayload {
    type PackError = Infallible;
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.instance_id.packed_len()
            + self.round.packed_len()
            + BEACON_PARTIAL_PUBLIC_KEY_LENGTH
            + BEACON_SIGNATURE_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.instance_id.pack(packer).map_err(PackError::infallible)?;
        self.round.pack(packer).map_err(PackError::infallible)?;

        // The size of `self.partial_public_key` is known to be 96 bytes.
        let partial_pk_bytes: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH] =
            self.partial_public_key.to_vec().try_into().unwrap();
        partial_pk_bytes.pack(packer).map_err(PackError::infallible)?;

        // The size of `self.partial_signature` is known to be 96 bytes.
        let partial_sig_bytes: [u8; BEACON_SIGNATURE_LENGTH] = self.partial_signature.to_vec().try_into().unwrap();
        partial_sig_bytes.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let instance_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let round = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let partial_public_key = <[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]>::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .into();
        let partial_signature = <[u8; BEACON_SIGNATURE_LENGTH]>::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .into();

        Ok(Self {
            version,
            instance_id,
            round,
            partial_public_key,
            partial_signature,
        })
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
        self.version.replace(version);
        self
    }

    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id.replace(instance_id);
        self
    }

    pub fn with_round(mut self, round: u64) -> Self {
        self.round.replace(round);
        self
    }

    pub fn with_partial_public_key(mut self, partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]) -> Self {
        self.partial_public_key.replace(partial_public_key);
        self
    }

    pub fn with_partial_signature(mut self, partial_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.partial_signature.replace(partial_signature);
        self
    }

    pub fn finish(self) -> Result<BeaconPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingField("round"))?;
        let partial_public_key = self
            .partial_public_key
            .ok_or(ValidationError::MissingField("partial_public_key"))?
            .into();
        let partial_signature = self
            .partial_signature
            .ok_or(ValidationError::MissingField("partial_signature"))?
            .into();

        Ok(BeaconPayload {
            version,
            instance_id,
            round,
            partial_public_key,
            partial_signature,
        })
    }
}
