// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::{INPUT_OUTPUT_COUNT_RANGE, IOTA_SUPPLY}, error::{MessageUnpackError, ValidationError}, input::Input, output::Output, payload::{Payload, PayloadPackError}, prelude::{SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput}};

use bee_ord::is_sorted;
use bee_packable::{
    error::UnpackPrefixError, PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker,
    VecPrefix,
};

use alloc::vec::Vec;
use core::{convert::Infallible, fmt};

/// Length (in bytes) of Transaction Essence pledge IDs (node IDs relating to pledge mana).
pub const PLEDGE_ID_LENGTH: usize = 32;

#[derive(Debug)]
pub enum TransactionEssencePackError {
    OptionalPayload(PayloadPackError),
}

impl_wrapped_variant!(TransactionEssencePackError, PayloadPackError, TransactionEssencePackError::OptionalPayload);
impl_from_infallible!(TransactionEssencePackError);

impl fmt::Display for TransactionEssencePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OptionalPayload(e) => write!(f, "Error packing payload: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum TransactionEssenceUnpackError {
    InvalidInputPrefixLength,
    InvalidOutputKind(u8),
    InvalidOutputPrefixLength,
    InvalidOptionTag(u8),
    ValidationError(ValidationError),
}

impl_from_infallible!(TransactionEssenceUnpackError);

impl From<UnpackPrefixError<UnknownTagError<u8>, u32>> for TransactionEssenceUnpackError {
    fn from(error: UnpackPrefixError<UnknownTagError<u8>, u32>) -> Self {
        match error {
            UnpackPrefixError::Packable(error) => match error {
                UnknownTagError(tag) => Self::InvalidOutputKind(tag),
            }
            UnpackPrefixError::Prefix(_) => Self::InvalidOutputPrefixLength, 
        }
    }
}

impl fmt::Display for TransactionEssenceUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInputPrefixLength => write!(f, "Invalid input prefix length"),
            Self::InvalidOutputKind(kind) => write!(f, "Invalid output kind: {}", kind),
            Self::InvalidOutputPrefixLength => write!(f, "Invalid output prefix length"),
            Self::InvalidOptionTag(tag) => write!(f, "Invalid tag for Option: {} is not 0 or 1", tag),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    inputs: Vec<Input>,
    /// Collection of transaction [Output]s.
    outputs: Vec<Output>,
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

impl Packable for TransactionEssence {
    type PackError = TransactionEssencePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.timestamp.packed_len()
            + self.access_pledge_id.packed_len()
            + self.consensus_pledge_id.packed_len()
            + VecPrefix::<Input, u32>::from(self.inputs.clone()).packed_len()
            + VecPrefix::<Output, u32>::from(self.outputs.clone()).packed_len()
            + self.payload.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.timestamp.pack(packer).map_err(PackError::infallible)?;
        self.access_pledge_id.pack(packer).map_err(PackError::infallible)?;
        self.consensus_pledge_id.pack(packer).map_err(PackError::infallible)?;
        self.inputs.pack(packer).map_err(PackError::coerce)?;
        self.outputs.pack(packer).map_err(PackError::coerce)?;
        self.payload.pack(packer).map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let timestamp = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let access_pledge_id = <[u8; PLEDGE_ID_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;
        let consensus_pledge_id = <[u8; PLEDGE_ID_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        // Inputs syntactical validation
        let inputs = VecPrefix::<Input, u32>::unpack(unpacker);

        let inputs_vec: Vec<Input> = if let Err(unpack_err) = inputs {
            match unpack_err {
                UnpackError::Packable(e) => match e {
                    UnpackPrefixError::Packable(err) => return Err(UnpackError::Packable(err)),
                    UnpackPrefixError::Prefix(_) => return Err(
                        UnpackError::Packable(TransactionEssenceUnpackError::InvalidInputPrefixLength.into())
                    ),
                }
                UnpackError::Unpacker(e) => return Err(UnpackError::Unpacker(e)),
            }
        } else {
            inputs.ok().unwrap().into()
        };

        validate_input_count(inputs_vec.len()).map_err(|e| UnpackError::Packable(e.into()))?;
        validate_inputs_unique_utxos(&inputs_vec).map_err(|e| UnpackError::Packable(e.into()))?;
        validate_inputs_sorted(&inputs_vec).map_err(|e| UnpackError::Packable(e.into()))?;

        // Outputs syntactical validation
        let outputs = VecPrefix::<Output, u32>::unpack(unpacker);

        let outputs_vec: Vec<Output> = if let Err(unpack_err) = outputs {
            match unpack_err {
                UnpackError::Packable(e) => match e {
                    UnpackPrefixError::Packable(err) => return Err(UnpackError::Packable(err)),
                    UnpackPrefixError::Prefix(_) => return Err(
                        UnpackError::Packable(TransactionEssenceUnpackError::InvalidOutputPrefixLength.into())
                    ),
                }
                UnpackError::Unpacker(e) => return Err(UnpackError::Unpacker(e)),
            }
        } else {
            outputs.ok().unwrap().into()
        };

        validate_output_count(outputs_vec.len()).map_err(|e| UnpackError::Packable(e.into()))?;
        validate_output_total(
            outputs_vec.iter().try_fold(0u64, |total, output| {
                let amount = validate_output_variant(output, &outputs_vec)?;
                total.checked_add(amount)
                    .ok_or_else(|| ValidationError::InvalidAccumulatedOutput(total as u128 + amount as u128))
            }).map_err(|e| UnpackError::Packable(e.into()))?
        ).map_err(|e| UnpackError::Packable(e.into()))?;
        validate_outputs_sorted(&outputs_vec).map_err(|e| UnpackError::Packable(e.into()))?;

        let payload = Option::<Payload>::unpack(unpacker).map_err(UnpackError::coerce)?;
        validate_payload(&payload).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self {
            version,
            timestamp,
            access_pledge_id,
            consensus_pledge_id,
            inputs: inputs_vec,
            outputs: outputs_vec,
            payload,
        })
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
    pub fn finish(self) -> Result<TransactionEssence, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let timestamp = self.timestamp.ok_or(ValidationError::MissingField("timestamp"))?;
        let access_pledge_id = self
            .access_pledge_id
            .ok_or(ValidationError::MissingField("access_pledge_id"))?;
        let consensus_pledge_id = self
            .consensus_pledge_id
            .ok_or(ValidationError::MissingField("consensus_pledge_id"))?;

        // Inputs syntactical validation
        validate_input_count(self.inputs.len())?;
        validate_inputs_unique_utxos(&self.inputs)?;
        validate_inputs_sorted(&self.inputs)?;

        // Outputs syntactical validation
        validate_output_count(self.outputs.len())?;
        validate_output_total(
            self.outputs.iter().try_fold(0u64, |total, output| {
                let amount = validate_output_variant(output, &self.outputs)?;
                total.checked_add(amount)
                    .ok_or_else(|| ValidationError::InvalidAccumulatedOutput(total as u128 + amount as u128))
            })?
        )?;
        validate_outputs_sorted(&self.outputs)?;

        validate_payload(&self.payload)?;

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

fn validate_input_count(len: usize) -> Result<(), ValidationError> {
    if !INPUT_OUTPUT_COUNT_RANGE.contains(&len) {
        Err(ValidationError::InvalidInputCount(len))
    } else {
        Ok(())
    }
}

fn validate_inputs_unique_utxos(inputs: &[Input]) -> Result<(), ValidationError> {
    for input in inputs.iter() {
        match input {
            Input::Utxo(u) => {
                if inputs.iter().filter(|i| *i == input).count() > 1 {
                    return Err(ValidationError::DuplicateUtxo(u.clone()));
                }
            }
        }
    }

    Ok(())
}

fn validate_inputs_sorted(inputs: &[Input]) -> Result<(), ValidationError> {
    if !is_sorted(inputs.iter().map(Packable::pack_to_vec)) {
        Err(ValidationError::TransactionInputsNotSorted)
    } else {
        Ok(())
    }
}

fn validate_output_count(len: usize) -> Result<(), ValidationError> {
    if !INPUT_OUTPUT_COUNT_RANGE.contains(&len) {
        Err(ValidationError::InvalidOutputCount(len))
    } else {
        Ok(())
    }
}

fn validate_output_variant(output: &Output, outputs: &[Output]) -> Result<u64, ValidationError> {
    match output {
        Output::SignatureLockedSingle(single) => validate_single(single, outputs),
        Output::SignatureLockedDustAllowance(dust_allowance) => validate_dust_allowance(dust_allowance, outputs),
    }
}

fn validate_single(single: &SignatureLockedSingleOutput, outputs: &[Output]) -> Result<u64, ValidationError> {
    if outputs
        .iter()
        .filter(|o| matches!(o, Output::SignatureLockedSingle(s) if s.address() == single.address()))
        .count()
        > 1
    {
        Err(ValidationError::DuplicateAddress(*single.address()))
    } else {
        Ok(single.amount())
    }
}

fn validate_dust_allowance(
    dust_allowance: &SignatureLockedDustAllowanceOutput, 
    outputs: &[Output]
) -> Result<u64, ValidationError> {
    if outputs
        .iter()
        .filter(|o| matches!(o, Output::SignatureLockedDustAllowance(s) if s.address() == dust_allowance.address()))
        .count()
        > 1
    {
        Err(ValidationError::DuplicateAddress(*dust_allowance.address()))
    } else {
        Ok(dust_allowance.amount())
    }
}

fn validate_output_total(total: u64) -> Result<(), ValidationError> {
    if total > IOTA_SUPPLY {
        Err(ValidationError::InvalidAccumulatedOutput(total as u128))
    } else {
        Ok(())
    }
}

fn validate_outputs_sorted(outputs: &[Output]) -> Result<(), ValidationError> {
    if !is_sorted(outputs.iter().map(Packable::pack_to_vec)) {
        Err(ValidationError::TransactionOutputsNotSorted)
    } else {
        Ok(())
    }
}

fn validate_payload(payload: &Option<Payload>) -> Result<(), ValidationError> {
    if !matches!(*payload, None | Some(Payload::Indexation(_))) {
        // Unwrap is fine because we just checked that the Option is not None.
        Err(ValidationError::InvalidPayloadKind(payload.as_ref().unwrap().kind()))
    } else {
        Ok(())
    }
}
