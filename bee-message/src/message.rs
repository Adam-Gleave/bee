// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parents::Parents,
    // payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    payload::Payload,
    Error, MessageId,
};

// use bee_common::packable::{Packable, Read, Write};
use bee_pow::providers::{miner::Miner, NonceProvider, NonceProviderBuilder};

use crypto::{hashes::{blake2b::Blake2b256, Digest}, signatures::ed25519};

const MESSAGE_VERSION: u8 = 1;

/// The minimum number of bytes in a message.
pub const MESSAGE_LENGTH_MIN: usize = 53;

/// The maximum number of bytes in a message.
pub const MESSAGE_LENGTH_MAX: usize = 32768;

/// Length (in bytes) of a public key.
pub const MESSAGE_PUBLIC_KEY_LENGTH: usize = 32;

/// Length (in bytes) of a message signature.
pub const MESSAGE_SIGNATURE_LENGTH: usize = 64;

const DEFAULT_POW_SCORE: f64 = 4000f64;
const DEFAULT_NONCE: u64 = 0;

/// Represent the object that nodes gossip around the network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// Specifies the version of the message structure.
    version: u8,
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

    // /// Computes the identifier of the message.
    // pub fn id(&self) -> (MessageId, Vec<u8>) {
    //     let bytes = self.pack_new();
    //     let id = Blake2b256::digest(&bytes);

    //     (MessageId::new(id.into()), bytes)
    // }

    /// Returns the version of a `Message`.
    pub fn version(&self) -> u8 {
        self.version
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

    // /// Hashes the `Message` contents, excluding the signature.
    // pub fn hash(&self) -> [u8; 32] {
    //     let mut message_bytes = self.pack_new();
    //     message_bytes = message_bytes[..message_bytes.len() - core::mem::size_of::<u64>()].to_vec();

    //     Blake2b256::digest(&message_bytes).into()
    // }

    /// Verifies the `Message` signature against the contents of the `Message`.
    pub fn verify(&self) -> Result<(), Error> {
        let ed25519_public_key =
            ed25519::PublicKey::from_compressed_bytes(self.issuer_public_key)?;

        // Unwrapping is okay here, since the length of the signature is already known to be correct.
        let ed25519_signature = ed25519::Signature::from_bytes(self.signature);

        // let hash = self.hash();

        // if !ed25519_public_key.verify(&ed25519_signature, &hash) {
        //     Err(Error::InvalidSignature)      
        // } else {
        Ok(())
        // }
    }
}

// impl Packable for Message {
//     type Error = Error;

//     fn packed_len(&self) -> usize {
//         self.version.packed_len()
//             + self.strong_parents.packed_len()
//             + self.weak_parents.packed_len()
//             + MESSAGE_PUBLIC_KEY_LENGTH
//             + self.issue_timestamp.packed_len()
//             + self.sequence_number.packed_len()
//             + option_payload_packed_len(self.payload.as_ref())
//             + self.nonce.packed_len()
//             + MESSAGE_SIGNATURE_LENGTH
//     }

//     fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
//         // FIXME Strong/weak parents length encoding
//         self.version.pack(writer)?;
//         self.strong_parents.pack(writer)?;
//         self.weak_parents.pack(writer)?;
//         self.issuer_public_key.pack(writer)?;
//         self.issue_timestamp.pack(writer)?;
//         self.sequence_number.pack(writer)?;
//         option_payload_pack(writer, self.payload.as_ref())?;
//         self.nonce.pack(writer)?;
//         self.signature.pack(writer)?;

//         Ok(())
//     }

//     fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
//         // FIXME Strong/weak parents length encoding
//         let version = u8::unpack_inner::<R, CHECK>(reader)?;
//         let strong_parents = Parents::unpack_inner::<R, CHECK>(reader)?;
//         let weak_parents = Parents::unpack_inner::<R, CHECK>(reader)?;
//         let issuer_public_key = <[u8; MESSAGE_PUBLIC_KEY_LENGTH]>::unpack_inner::<R, CHECK>(reader)?;
//         let issue_timestamp = u64::unpack_inner::<R, CHECK>(reader)?;
//         let sequence_number = u32::unpack_inner::<R, CHECK>(reader)?;
//         let (payload_len, payload) = option_payload_unpack::<R, CHECK>(reader)?;

//         if CHECK
//             && !matches!(
//                 payload,
//                 None | Some(Payload::Transaction(_)) | Some(Payload::Indexation(_))
//             )
//         {
//             // Safe to unwrap, since it's known not to be None.
//             return Err(Error::InvalidPayloadKind(payload.unwrap().kind()));
//         }

//         let nonce = u64::unpack_inner::<R, CHECK>(reader)?;
//         let signature = <[u8; MESSAGE_SIGNATURE_LENGTH]>::unpack_inner::<R, CHECK>(reader)?;

//         // Computed instead of calling `packed_len` on Self because `payload_len` is already known, and it may be
//         // expensive to call `payload.packed_len()` twice.
//         let message_len = version.packed_len()
//             + strong_parents.packed_len()
//             + weak_parents.packed_len()
//             + MESSAGE_PUBLIC_KEY_LENGTH
//             + issue_timestamp.packed_len()
//             + sequence_number.packed_len()
//             + payload_len
//             + nonce.packed_len()
//             + MESSAGE_SIGNATURE_LENGTH;
        
//         if CHECK && message_len > MESSAGE_LENGTH_MAX {
//             return Err(Error::InvalidMessageLength(message_len));
//         }

//         // When parsing the message is complete, there should not be any trailing bytes left that were not parsed.
//         if CHECK && reader.bytes().next().is_some() {
//             return Err(Error::RemainingBytesAfterMessage);
//         }

//         Ok(Self {
//             version,
//             strong_parents,
//             weak_parents,
//             issuer_public_key,
//             issue_timestamp,
//             sequence_number,
//             payload,
//             nonce,
//             signature,
//         })
//     }
// }

/// A builder to build a `Message`.
pub struct MessageBuilder<P: NonceProvider = Miner> {
    version: u8,
    strong_parents: Option<Parents>,
    weak_parents: Option<Parents>,
    issuer_public_key: Option<[u8; MESSAGE_PUBLIC_KEY_LENGTH]>,
    issue_timestamp: Option<u64>,
    sequence_number: Option<u32>,
    payload: Option<Payload>,
    nonce_provider: Option<(P, f64)>,
    signature: Option<[u8; MESSAGE_SIGNATURE_LENGTH]>,
}

impl<P: NonceProvider> Default for MessageBuilder<P> {
    fn default() -> Self {
        Self {
            version: MESSAGE_VERSION, 
            strong_parents: None,
            weak_parents: None,
            issuer_public_key: None,
            issue_timestamp: None,
            sequence_number: None,
            payload: None,
            nonce_provider: None,
            signature: None,
        }
    }
}

impl<P: NonceProvider> MessageBuilder<P> {
    /// Creates a new `MessageBuilder`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a version number to a `MessageBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version = version;
        self
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
    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score));
        self
    }

    /// Adds a signature to a `MessageBuilder`.
    pub fn with_signature(mut self, signature: [u8; MESSAGE_SIGNATURE_LENGTH]) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Finished the `MessageBuilder`, consuming it to build a `Message`.
    pub fn finish(self) -> Result<Message, Error> {
        let version = self.version;
        let strong_parents = self.strong_parents.ok_or(Error::MissingField("strong_parents"))?;
        let weak_parents = self.weak_parents.ok_or(Error::MissingField("weak_parents"))?;
        let issuer_public_key = self.issuer_public_key.ok_or(Error::MissingField("issuer_public_key"))?;
        let issue_timestamp = self.issue_timestamp.ok_or(Error::MissingField("issue_timestap"))?;
        let sequence_number = self.sequence_number.ok_or(Error::MissingField("sequence_number"))?;

        // FIXME payload types
        if !matches!(
            self.payload,
            None | Some(Payload::Transaction(_)) | Some(Payload::Indexation(_))
        ) {
            // Safe to unwrap here, since it's known not to be None.
            return Err(Error::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        let signature = self.signature.ok_or(Error::MissingField("signature"))?;

        let mut message = Message {
            version,
            strong_parents,
            weak_parents,
            issuer_public_key,
            issue_timestamp,
            sequence_number,
            payload: self.payload,
            nonce: 0,
            signature,
        };

        // let message_bytes = message.pack_new();

        // if message_bytes.len() > MESSAGE_LENGTH_MAX {
        //     return Err(Error::InvalidMessageLength(message_bytes.len()));
        // }

        let (nonce_provider, _target_score) = self
            .nonce_provider
            .unwrap_or((P::Builder::new().finish(), DEFAULT_POW_SCORE));

        message.nonce = DEFAULT_NONCE;

        // message.nonce = nonce_provider
        //     .nonce(
        //         &message_bytes[..message_bytes.len() - (core::mem::size_of::<u64>() + MESSAGE_SIGNATURE_LENGTH)],
        //         target_score,
        //     )
        //     .unwrap_or(DEFAULT_NONCE);

        Ok(message)
    }
}
