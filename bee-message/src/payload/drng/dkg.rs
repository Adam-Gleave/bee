// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::{MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{convert::Infallible, fmt};

#[derive(Debug)]
pub enum DkgPackError {
    InvalidPrefixLength,
}

impl From<PackPrefixError<Infallible, u32>> for DkgPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefixLength,
        }
    }
}

impl fmt::Display for DkgPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for encrypted deal data"),
        }
    }
}

#[derive(Debug)]
pub enum DkgUnpackError {
    InvalidPrefixLength,
}

impl_from_infallible!(DkgUnpackError);

impl From<UnpackPrefixError<Infallible, u32>> for DkgUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefixLength,
        }
    }
}

impl fmt::Display for DkgUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for salt bytes"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EncryptedDeal {
    dh_key: Vec<u8>,
    nonce: Vec<u8>,
    encrypted_share: Vec<u8>,
    threshold: u32,
    commitments: Vec<u8>,
}

impl EncryptedDeal {
    pub fn builder() -> EncryptedDealBuilder {
        EncryptedDealBuilder::new()
    }

    pub fn dh_key(&self) -> &[u8] {
        self.dh_key.as_slice()
    }

    pub fn nonce(&self) -> &[u8] {
        self.nonce.as_slice()
    }

    pub fn encrypted_share(&self) -> &[u8] {
        self.encrypted_share.as_slice()
    }

    pub fn threshold(&self) -> u32 {
        self.threshold
    }

    pub fn commitments(&self) -> &[u8] {
        self.commitments.as_slice()
    }
}

impl Packable for EncryptedDeal {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        VecPrefix::<u8, u32>::from(self.dh_key.clone()).packed_len()
            + VecPrefix::<u8, u32>::from(self.nonce.clone()).packed_len()
            + VecPrefix::<u8, u32>::from(self.encrypted_share.clone()).packed_len()
            + self.threshold.packed_len()
            + VecPrefix::<u8, u32>::from(self.commitments.clone()).packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        let prefixed_dh_key: VecPrefix<u8, u32> = self.dh_key.clone().into();
        prefixed_dh_key
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        let prefixed_nonce: VecPrefix<u8, u32> = self.nonce.clone().into();
        prefixed_nonce
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        let prefixed_encrypted_share: VecPrefix<u8, u32> = self.encrypted_share.clone().into();
        prefixed_encrypted_share
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        self.threshold.pack(packer).map_err(PackError::infallible)?;

        let prefixed_commitments: VecPrefix<u8, u32> = self.commitments.clone().into();
        prefixed_commitments
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let dh_key = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let nonce = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let encrypted_share = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let threshold = u32::unpack(unpacker).map_err(UnpackError::infallible)?;

        let commitments = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        Ok(Self {
            dh_key,
            nonce,
            encrypted_share,
            threshold,
            commitments,
        })
    }
}

#[derive(Default)]
pub struct EncryptedDealBuilder {
    dh_key: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    encrypted_share: Option<Vec<u8>>,
    threshold: Option<u32>,
    commitments: Option<Vec<u8>>,
}

impl EncryptedDealBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dh_key(mut self, dh_key: Vec<u8>) -> Self {
        self.dh_key.replace(dh_key);
        self
    }

    pub fn with_nonce(mut self, nonce: Vec<u8>) -> Self {
        self.nonce.replace(nonce);
        self
    }

    pub fn with_encrypted_share(mut self, encrypted_share: Vec<u8>) -> Self {
        self.encrypted_share.replace(encrypted_share);
        self
    }

    pub fn with_threshold(mut self, threshold: u32) -> Self {
        self.threshold.replace(threshold);
        self
    }

    pub fn with_commitments(mut self, commitments: Vec<u8>) -> Self {
        self.commitments.replace(commitments);
        self
    }

    pub fn finish(self) -> Result<EncryptedDeal, ValidationError> {
        let dh_key = self.dh_key.ok_or(ValidationError::MissingField("dh_key"))?;
        let nonce = self.nonce.ok_or(ValidationError::MissingField("nonce"))?;
        let encrypted_share = self
            .encrypted_share
            .ok_or(ValidationError::MissingField("encrypted_share"))?;
        let threshold = self.threshold.ok_or(ValidationError::MissingField("threshold"))?;
        let commitments = self.commitments.ok_or(ValidationError::MissingField("commitments"))?;

        Ok(EncryptedDeal {
            dh_key,
            nonce,
            encrypted_share,
            threshold,
            commitments,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DkgPayload {
    version: u8,
    instance_id: u32,
    from_index: u32,
    to_index: u32,
    deal: EncryptedDeal,
}

impl DkgPayload {
    // TODO verify values with goshimmer.
    pub const KIND: u32 = 4;

    pub fn builder() -> DkgPayloadBuilder {
        DkgPayloadBuilder::new()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    pub fn from_index(&self) -> u32 {
        self.from_index
    }

    pub fn to_index(&self) -> u32 {
        self.to_index
    }

    pub fn deal(&self) -> &EncryptedDeal {
        &self.deal
    }
}

impl Packable for DkgPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.instance_id.packed_len()
            + self.from_index.packed_len()
            + self.to_index.packed_len()
            + self.deal.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.instance_id.pack(packer).map_err(PackError::infallible)?;
        self.from_index.pack(packer).map_err(PackError::infallible)?;
        self.to_index.pack(packer).map_err(PackError::infallible)?;
        self.deal.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let instance_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let from_index = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let to_index = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let deal = EncryptedDeal::unpack(unpacker)?;

        Ok(Self {
            version,
            instance_id,
            from_index,
            to_index,
            deal,
        })
    }
}

#[derive(Default)]
pub struct DkgPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    from_index: Option<u32>,
    to_index: Option<u32>,
    deal: Option<EncryptedDeal>,
}

impl DkgPayloadBuilder {
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

    pub fn with_from_index(mut self, from_index: u32) -> Self {
        self.from_index.replace(from_index);
        self
    }

    pub fn with_to_index(mut self, to_index: u32) -> Self {
        self.to_index.replace(to_index);
        self
    }

    pub fn with_deal(mut self, deal: EncryptedDeal) -> Self {
        self.deal.replace(deal);
        self
    }

    pub fn finish(self) -> Result<DkgPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let from_index = self.from_index.ok_or(ValidationError::MissingField("from_index"))?;
        let to_index = self.to_index.ok_or(ValidationError::MissingField("to_index"))?;
        let deal = self.deal.ok_or(ValidationError::MissingField("deal"))?;

        Ok(DkgPayload {
            version,
            instance_id,
            from_index,
            to_index,
            deal,
        })
    }
}
