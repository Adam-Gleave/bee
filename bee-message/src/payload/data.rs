// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MessagePackError, MessageUnpackError};

use bee_packable::{error::{PackPrefixError, UnpackPrefixError}, Packable, Packer, PackError, Unpacker, UnpackError, VecPrefix};

use alloc::vec::Vec;
use core::{fmt, convert::Infallible};

#[derive(Debug)]
pub enum DataPackError {
    InvalidPrefixLength,
}

impl From<PackPrefixError<Infallible, u32>> for DataPackError {
    fn from(_: PackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefixLength
    }
}

impl fmt::Display for DataPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for data"),
        }
    }
}

#[derive(Debug)]
pub enum DataUnpackError {
    InvalidPrefixLength,
}

impl From<UnpackPrefixError<Infallible, u32>> for DataUnpackError {
    fn from(_: UnpackPrefixError<Infallible, u32>) -> Self {
        Self::InvalidPrefixLength
    }
}

impl fmt::Display for DataUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefixLength => write!(f, "Invalid prefix length for data"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPayload {
    version: u8,
    data: Vec<u8>,
}

impl DataPayload {
    pub const KIND: u32 = 0;

    pub fn new(version: u8, data: Vec<u8>) -> Self {
        Self { version, data }
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl Packable for DataPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + VecPrefix::<u8, u32>::from(self.data.clone()).packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        
        let prefixed_data: VecPrefix::<u8, u32> = self.data.clone().into();
        prefixed_data.pack(packer).map_err(PackError::coerce::<DataPackError>).map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let data = VecPrefix::<u8, u32>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DataUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();
    
        Ok(Self { version, data })
    }
}
