// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_packable::Packable;

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EncryptedDeal {
    dh_key: Box<[u8]>,
    nonce: Box<[u8]>,
    encrypted_share: Box<[u8]>,
    threshold: u32,
    commitments: Box<[u8]>,
}

impl EncryptedDeal {
    pub fn dh_key(&self) -> &[u8] {
        &self.dh_key
    } 

    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }

    pub fn encrypted_share(&self) -> &[u8] {
        &self.encrypted_share
    }

    pub fn threshold(&self) -> u32 {
        self.threshold
    }

    pub fn commitments(&self) -> &[u8] {
        &self.commitments
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Packable)]
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
    pub const KIND: u32 = 7;

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

pub struct DkgPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    from_index: Option<u32>,
    to_index: Option<u32>,
    deal: Option<EncryptedDeal>,
}

impl DkgPayloadBuilder {
    pub fn new() -> Self {
        Self {
            version: None,
            instance_id: None,
            from_index: None,
            to_index: None,
            deal: None,
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

    pub fn with_from_index(mut self, from_index: u32) -> Self {
        self.from_index = Some(from_index);
        self
    }

    pub fn with_to_index(mut self, to_index: u32) -> Self {
        self.to_index = Some(to_index);
        self
    }

    pub fn with_deal(mut self, deal: EncryptedDeal) -> Self {
        self.deal = Some(deal);
        self
    }

    pub fn finish(self) -> Result<DkgPayload, Error> {
        let version = self.version.ok_or(Error::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(Error::MissingField("instance_id"))?;
        let from_index = self.from_index.ok_or(Error::MissingField("from_index"))?;
        let to_index = self.to_index.ok_or(Error::MissingField("to_index"))?;
        let deal = self.deal.ok_or(Error::MissingField("deal"))?;

        Ok(DkgPayload {
            version,
            instance_id,
            from_index,
            to_index,
            deal,
        })
    }
}