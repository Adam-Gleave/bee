// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{MessagePackError, MessageUnpackError, ValidationError},
    parents::Parents,
    payload::Payload,
    MessageId,
};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519,
};

/// The minimum number of bytes in a message.
pub const MESSAGE_LENGTH_MIN: usize = 53;

/// The maximum number of bytes in a message.
pub const MESSAGE_LENGTH_MAX: usize = 32768;

/// Length (in bytes) of a public key.
pub const MESSAGE_PUBLIC_KEY_LENGTH: usize = 32;

/// Length (in bytes) of a message signature.
pub const MESSAGE_SIGNATURE_LENGTH: usize = 64;

/// Represent the object that nodes gossip around the network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// Message [Parents] in which the past cone is "Liked".
    strong_parents: Parents,
    /// Message [Parents] in which the past cone is "Disliked", but the parents themselves are "Liked".
    weak_parents: Parents,
    /// The public key of the issuing node.
    issuer_public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH],
    /// The Unix timestamp at the moment of issue.
    issue_timestamp: u64,
    /// The sequence number of the message, indicating the marker sequence it belongs to.
    sequence_number: u32,
    /// The optional [Payload] of the message.
    payload: Option<Payload>,
    /// The result of the Proof of Work in order for the message to be accepted into the tangle.
    nonce: u64,
    /// Signature signing the above message fields.
    signature: [u8; MESSAGE_SIGNATURE_LENGTH],
}

impl Message {
    /// Creates a new `MessageBuilder` to construct an instance of a `Message`.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Computes the identifier of the message.
    pub fn id(&self) -> (MessageId, Vec<u8>) {
        let bytes = self.pack_to_vec().unwrap();

        let id = Blake2b256::digest(&bytes);

        (MessageId::new(id.into()), bytes)
    }

    /// Returns the strong parents of a `Message`.
    pub fn strong_parents(&self) -> &Parents {
        &self.strong_parents
    }

    /// Returns the weak parents of a `Message`.
    pub fn weak_parents(&self) -> &Parents {
        &self.weak_parents
    }

    /// Returns the `Message` issuer public key.
    pub fn issuer_public_key(&self) -> &[u8; MESSAGE_PUBLIC_KEY_LENGTH] {
        &self.issuer_public_key
    }

    /// Returns the `Message` issuance timestamp.
    pub fn issue_timestamp(&self) -> u64 {
        self.issue_timestamp
    }

    /// Returns the sequence number of a `Message`.
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    /// Returns the optional payload of a `Message`.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    /// Returns the nonce of a `Message`.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Returns the `Message` signature.
    pub fn signature(&self) -> &[u8; MESSAGE_SIGNATURE_LENGTH] {
        &self.signature
    }

    /// Hashes the `Message` contents, excluding the signature.
    pub fn hash(&self) -> [u8; 32] {
        let mut bytes = self.pack_to_vec().unwrap();

        bytes = bytes[..bytes.len() - core::mem::size_of::<u64>()].to_vec();

        Blake2b256::digest(&bytes).into()
    }

    /// Verifies the `Message` signature against the contents of the `Message`.
    pub fn verify(&self) -> Result<(), ValidationError> {
        let ed25519_public_key = ed25519::PublicKey::from_compressed_bytes(self.issuer_public_key)?;

        // Unwrapping is okay here, since the length of the signature is already known to be correct.
        let ed25519_signature = ed25519::Signature::from_bytes(self.signature);

        let hash = self.hash();

        if !ed25519_public_key.verify(&ed25519_signature, &hash) {
            Err(ValidationError::InvalidSignature)
        } else {
            Ok(())
        }
    }
}

impl Packable for Message {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.strong_parents.packed_len()
            + self.weak_parents.packed_len()
            + self.issuer_public_key.packed_len()
            + self.issue_timestamp.packed_len()
            + self.sequence_number.packed_len()
            + self.payload.packed_len()
            + self.nonce.packed_len()
            + self.signature.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.strong_parents.pack(packer).map_err(PackError::coerce)?;
        self.weak_parents.pack(packer).map_err(PackError::coerce)?;
        self.issuer_public_key.pack(packer).map_err(PackError::infallible)?;
        self.issue_timestamp.pack(packer).map_err(PackError::infallible)?;
        self.sequence_number.pack(packer).map_err(PackError::infallible)?;
        self.payload.pack(packer).map_err(PackError::coerce)?;
        self.nonce.pack(packer).map_err(PackError::infallible)?;
        self.signature.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let strong_parents = Parents::unpack(unpacker).map_err(UnpackError::coerce)?;
        let weak_parents = Parents::unpack(unpacker).map_err(UnpackError::coerce)?;
        let issuer_public_key = <[u8; MESSAGE_PUBLIC_KEY_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;
        let issue_timestamp = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let sequence_number = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let payload = Option::<Payload>::unpack(unpacker).map_err(UnpackError::coerce)?;
        let nonce = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let signature = <[u8; MESSAGE_SIGNATURE_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self {
            strong_parents,
            weak_parents,
            issuer_public_key,
            issue_timestamp,
            sequence_number,
            payload,
            nonce,
            signature,
        })
    }
}

/// A builder to build a `Message`.
#[derive(Default)]
pub struct MessageBuilder {
    strong_parents: Option<Parents>,
    weak_parents: Option<Parents>,
    issuer_public_key: Option<[u8; MESSAGE_PUBLIC_KEY_LENGTH]>,
    issue_timestamp: Option<u64>,
    sequence_number: Option<u32>,
    payload: Option<Payload>,
    nonce: Option<u64>,
    signature: Option<[u8; MESSAGE_SIGNATURE_LENGTH]>,
}

impl MessageBuilder {
    /// Creates a new `MessageBuilder`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds strong parents to a `MessageBuilder`.
    pub fn with_strong_parents(mut self, strong_parents: Parents) -> Self {
        self.strong_parents = Some(strong_parents);
        self
    }

    /// Adds weak parents to a `MessageBuilder`.
    pub fn with_weak_parents(mut self, weak_parents: Parents) -> Self {
        self.weak_parents = Some(weak_parents);
        self
    }

    /// Adds an issuer public key to a `MessageBuilder`.
    pub fn with_issuer_public_key(mut self, issuer_public_key: [u8; MESSAGE_PUBLIC_KEY_LENGTH]) -> Self {
        self.issuer_public_key = Some(issuer_public_key);
        self
    }

    /// Adds an issuance timestamp to a `MessageBuilder`.
    pub fn with_issue_timestamp(mut self, issue_timestamp: u64) -> Self {
        self.issue_timestamp = Some(issue_timestamp);
        self
    }

    /// Adds a sequence number to a `MessageBuilder`.
    pub fn with_sequence_number(mut self, sequence_number: u32) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    /// Adds a payload to a `MessageBuilder`.
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Adds a nonce provider to a `MessageBuilder`.
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Adds a signature to a `MessageBuilder`.
    pub fn with_signature(mut self, signature: [u8; MESSAGE_SIGNATURE_LENGTH]) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Finished the `MessageBuilder`, consuming it to build a `Message`.
    pub fn finish(self) -> Result<Message, ValidationError> {
        let strong_parents = self
            .strong_parents
            .ok_or(ValidationError::MissingField("strong_parents"))?;
        let weak_parents = self.weak_parents.ok_or(ValidationError::MissingField("weak_parents"))?;
        let issuer_public_key = self
            .issuer_public_key
            .ok_or(ValidationError::MissingField("issuer_public_key"))?;
        let issue_timestamp = self
            .issue_timestamp
            .ok_or(ValidationError::MissingField("issue_timestap"))?;
        let sequence_number = self
            .sequence_number
            .ok_or(ValidationError::MissingField("sequence_number"))?;

        // FIXME payload types
        if !matches!(
            self.payload,
            None | Some(Payload::Transaction(_)) | Some(Payload::Indexation(_))
        ) {
            // Safe to unwrap here, since it's known not to be None.
            return Err(ValidationError::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        let nonce = self.nonce.ok_or(ValidationError::MissingField("nonce"))?;
        let signature = self.signature.ok_or(ValidationError::MissingField("signature"))?;

        let message = Message {
            strong_parents,
            weak_parents,
            issuer_public_key,
            issue_timestamp,
            sequence_number,
            payload: self.payload,
            nonce,
            signature,
        };

        let bytes = message.pack_to_vec().unwrap();

        if bytes.len() > MESSAGE_LENGTH_MAX {
            return Err(ValidationError::InvalidMessageLength(bytes.len()));
        }

        Ok(message)
    }
}
