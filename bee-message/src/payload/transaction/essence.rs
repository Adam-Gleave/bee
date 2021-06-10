// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::{INPUT_OUTPUT_COUNT_RANGE, IOTA_SUPPLY},
    input::Input,
    output::Output,
    payload::Payload,
    Error,
};

use bee_ord::is_sorted;
use bee_packable::{Packable, UnknownTagError, UnpackOptionError, VecPrefix};

use alloc::vec::Vec;
use core::convert::Infallible;

/// Length (in bytes) of Transaction Essence pledge IDs (node IDs relating to pledge mana).
pub const PLEDGE_ID_LENGTH: usize = 32;

pub enum TransactionUnpackError {
    UnknownTagError,
    OptionError,
}

impl<T> From<UnknownTagError<T>> for TransactionUnpackError {
    fn from(_: UnknownTagError<T>) -> Self {
        Self::UnknownTagError
    }
}

impl<T> From<UnpackOptionError<T>> for TransactionUnpackError {
    fn from(_: UnpackOptionError<T>) -> Self {
        Self::OptionError
    }
}

impl From<Infallible> for TransactionUnpackError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(error = TransactionUnpackError)]
pub struct TransactionEssence {
    /// Transaction essence version number.
    version: u8,
    /// Timestamp of the transaction.
    timestamp: u64,
    /// Node ID to which the access mana of the transaction is pledged.
    access_pledge_id: [u8; PLEDGE_ID_LENGTH],
    /// Node ID to which the consensus mana of the transaction is pledged.
    consensus_pledge_id: [u8; PLEDGE_ID_LENGTH],
    /// Collection of transaction [Input]s.
    inputs: VecPrefix<Input, u16>,
    /// Collection of transaction [Output]s.
    outputs: VecPrefix<Output, u16>,
    /// Optional additional payload.
    payload: Option<Payload>,
}

impl TransactionEssence {
    /// Create a new `TransactionEssenceBuilder` to build a `TransactionEssence`.
    pub fn builder() -> TransactionEssenceBuilder {
        TransactionEssenceBuilder::new()
    }

    /// Return the version number of a Transaction Essence.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Return the timestamp of a Transaction Essence.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Return the node ID to which the access mana of the transaction is pledged.
    pub fn access_pledge_id(&self) -> &[u8; PLEDGE_ID_LENGTH] {
        &self.access_pledge_id
    }

    /// Return the node ID to which the consensus mana of the transaction is pledged.
    pub fn consensus_pledge_id(&self) -> &[u8; PLEDGE_ID_LENGTH] {
        &self.consensus_pledge_id
    }

    /// Return the inputs of a `TransactionEssence`.
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Return the outputs of a `TransactionEssence`.
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Return the optional payload of a `TransactionEssence`.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

/// A builder to build a `TransactionEssence`.
#[derive(Debug, Default)]
pub struct TransactionEssenceBuilder {
    version: Option<u8>,
    timestamp: Option<u64>,
    access_pledge_id: Option<[u8; PLEDGE_ID_LENGTH]>,
    consensus_pledge_id: Option<[u8; PLEDGE_ID_LENGTH]>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl TransactionEssenceBuilder {
    /// Creates a new `TransactionEssenceBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a version number to a `TransactionEssenceBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    /// Adds a timestamp to a `TransactionEssenceBuilder`.
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Adds an access pledge ID to a `TransactionEssenceBuilder`.
    pub fn with_access_pledge_id(mut self, access_pledge_id: [u8; PLEDGE_ID_LENGTH]) -> Self {
        self.access_pledge_id = Some(access_pledge_id);
        self
    }

    /// Adds a consensus pledge ID to a `TransactionEssenceBuilder`.
    pub fn with_consensus_pledge_id(mut self, consensus_pledge_id: [u8; PLEDGE_ID_LENGTH]) -> Self {
        self.consensus_pledge_id = Some(consensus_pledge_id);
        self
    }

    /// Adds inputs to a `TransactionEssenceBuilder`
    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Add an input to a `TransactionEssenceBuilder`.
    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add outputs to a `TransactionEssenceBuilder`.
    pub fn with_outputs(mut self, outputs: Vec<Output>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Add an output to a `TransactionEssenceBuilder`.
    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    /// Add a payload to a `TransactionEssenceBuilder`.
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Finishes a `TransactionEssenceBuilder` into a `TransactionEssence`.
    pub fn finish(self) -> Result<TransactionEssence, Error> {
        let version = self.version.ok_or(Error::MissingField("version"))?;
        let timestamp = self.timestamp.ok_or(Error::MissingField("timestamp"))?;
        let access_pledge_id = self.access_pledge_id.ok_or(Error::MissingField("access_pledge_id"))?;
        let consensus_pledge_id = self.consensus_pledge_id.ok_or(Error::MissingField("consensus_pledge_id"))?;

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.inputs.len()) {
            return Err(Error::InvalidInputOutputCount(self.inputs.len()));
        }

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.outputs.len()) {
            return Err(Error::InvalidInputOutputCount(self.outputs.len()));
        }

        if !matches!(self.payload, None | Some(Payload::Indexation(_))) {
            // Unwrap is fine because we just checked that the Option is not None.
            return Err(Error::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        for input in self.inputs.iter() {
            match input {
                Input::Utxo(u) => {
                    if self.inputs.iter().filter(|i| *i == input).count() > 1 {
                        return Err(Error::DuplicateUtxo(u.clone()));
                    }
                }
            }
        }

        // Inputs must be lexicographically sorted in their serialised forms.
        if !is_sorted(self.inputs.iter().map(Packable::pack_new)) {
            return Err(Error::TransactionInputsNotSorted);
        }

        let mut total: u64 = 0;

        for output in self.outputs.iter() {
            match output {
                Output::SignatureLockedSingle(single) => {
                    // The addresses must be unique in the set of SignatureLockedSingleOutputs.
                    if self
                        .outputs
                        .iter()
                        .filter(|o| matches!(o, Output::SignatureLockedSingle(s) if s.address() == single.address()))
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateAddress(*single.address()));
                    }

                    total = total
                        .checked_add(single.amount())
                        .ok_or_else(|| Error::InvalidAccumulatedOutput((total + single.amount()) as u128))?;
                }
                Output::SignatureLockedDustAllowance(dust_allowance) => {
                    // The addresses must be unique in the set of SignatureLockedDustAllowanceOutputs.
                    if self
                        .outputs
                        .iter()
                        .filter(
                            |o| matches!(o, Output::SignatureLockedDustAllowance(s) if s.address() == dust_allowance.address()),
                        )
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateAddress(*dust_allowance.address()));
                    }

                    total = total.checked_add(dust_allowance.amount()).ok_or_else(|| {
                        Error::InvalidAccumulatedOutput(total as u128 + dust_allowance.amount() as u128)
                    })?;
                }
            }

            // Accumulated output balance must not exceed the total supply of tokens.
            if total > IOTA_SUPPLY {
                return Err(Error::InvalidAccumulatedOutput(total as u128));
            }
        }

        // Outputs must be lexicographically sorted in their serialised forms.
        if !is_sorted(self.outputs.iter().map(Packable::pack_new)) {
            return Err(Error::TransactionOutputsNotSorted);
        }

        Ok(TransactionEssence {
            version,
            timestamp,
            access_pledge_id,
            consensus_pledge_id,
            inputs: self.inputs.into(),
            outputs: self.outputs.into(),
            payload: self.payload,
        })
    }
}
