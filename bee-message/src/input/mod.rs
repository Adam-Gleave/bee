// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod utxo;

pub use utxo::UtxoInput;

use bee_packable::Packable;

/// A generic input supporting different input kinds.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8)]
pub enum Input {
    /// A UTXO input.
    #[packable(tag = 0)]
    Utxo(UtxoInput),
}

impl Input {
    /// Returns the input kind of an `Input`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Utxo(_) => UtxoInput::KIND,
        }
    }
}

impl From<UtxoInput> for Input {
    fn from(input: UtxoInput) -> Self {
        Self::Utxo(input)
    }
}
