// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::{Ed25519Address, ED25519_ADDRESS_LENGTH};

use crate::{signature::SignatureUnlock, Error};

use bee_common::packable::{Packable, Packer, UnknownTagError, Unpacker, UnpackError};

use bech32::{self, FromBase32, ToBase32, Variant};

use alloc::{str::FromStr, string::String};
use core::convert::TryFrom;

/// A generic address supporting different address kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8)]
pub enum Address {
    /// An Ed25519 address.
    #[packable(tag = 0)]
    Ed25519(Ed25519Address),
}

impl Address {
    /// Returns the address kind of an `Address`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Address::KIND,
        }
    }

    // /// Tries to create an `Address` from a Bech32 encoded string.
    // pub fn try_from_bech32(addr: &str) -> Result<Self, Error> {
    //     match bech32::decode(addr) {
    //         Ok((_hrp, data, _)) => {
    //             let bytes = Vec::<u8>::from_base32(&data).map_err(|_| Error::InvalidAddress)?;
    //             Self::unpack(&mut bytes.as_slice()).map_err(|_| Error::InvalidAddress)
    //         }
    //         Err(_) => Err(Error::InvalidAddress),
    //     }
    // }

    // /// Encodes this address to a Bech32 string with the hrp (human readable part) argument as prefix.
    // pub fn to_bech32(&self, hrp: &str) -> String {
    //     bech32::encode(hrp, self.pack_new().to_base32(), Variant::Bech32).expect("Invalid address.")
    // }

    /// Verifies a [`SignatureUnlock`] for a message against the [`Address`].
    pub fn verify(&self, msg: &[u8], signature: &SignatureUnlock) -> Result<(), Error> {
        match self {
            Address::Ed25519(address) => {
                let SignatureUnlock::Ed25519(signature) = signature;
                address.verify(msg, signature)
            }
        }
    }
}

impl From<Ed25519Address> for Address {
    fn from(address: Ed25519Address) -> Self {
        Self::Ed25519(address)
    }
}
