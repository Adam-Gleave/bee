// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, signature::ED25519_PUBLIC_KEY_LENGTH};

use bee_packable::{error::{PackPrefixError, UnpackPrefixError}, Packable, Packer, PackError, Unpacker, UnpackError, VecPrefix};

use core::{fmt, convert::Infallible};

#[derive(Debug)]
pub enum SaltDeclarationPackError {
    InvalidPrefixLength,
}

impl From<PackPrefixError<Infallible, u32>> for SaltDeclarationPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefixLength,
        }
    }
}

impl fmt::Display for SaltDeclarationPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for salt bytes"),
        }
    }
}

#[derive(Debug)]
pub enum SaltDeclarationUnpackError {
    InvalidPrefixLength,
}

impl From<UnpackPrefixError<Infallible, u32>> for SaltDeclarationUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefixLength,
        }
    }
}

impl From<Infallible> for SaltDeclarationUnpackError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

impl fmt::Display for SaltDeclarationUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for salt bytes"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Salt {
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

impl Packable for Salt {
    type PackError = SaltDeclarationPackError;
    type UnpackError = SaltDeclarationUnpackError;

    fn packed_len(&self) -> usize {
        0u32.packed_len()
            + self.bytes.packed_len()
            + self.expiry_time.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        let prefixed_bytes: VecPrefix<u8, u32> = self.bytes.clone().into();
        prefixed_bytes.pack(packer).map_err(PackError::coerce)?;

        self.expiry_time.pack(packer).map_err(PackError::infallible)?;


        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let bytes = VecPrefix::<u8, u32>::unpack(unpacker).map_err(UnpackError::coerce)?.into();
        let expiry_time = u64::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self { bytes, expiry_time })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl Packable for SaltDeclarationPayload {
    type PackError = SaltDeclarationPackError;
    type UnpackError = SaltDeclarationUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.node_id.packed_len()
            + self.salt.packed_len()
            + self.timestamp.packed_len()
            + self.signature.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.node_id.pack(packer).map_err(PackError::infallible)?;
        self.salt.pack(packer).map_err(PackError::coerce)?;
        self.timestamp.pack(packer).map_err(PackError::infallible)?;
        self.signature.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let node_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let salt = Salt::unpack(unpacker).map_err(UnpackError::coerce)?;
        let timestamp = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let signature = <[u8; ED25519_PUBLIC_KEY_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self {
            version,
            node_id,
            salt,
            timestamp,
            signature,
        })
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
