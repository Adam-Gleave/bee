// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Opinion};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    payload::transaction::{TransactionId, TRANSACTION_ID_LENGTH},
    MessageId,
    MESSAGE_ID_LENGTH,
};

/// Holds a conflicting transaction ID and its opinion.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Conflict {
    /// Conflicting transaction ID.
    pub id: TransactionId,
    /// Opinion of the conflict.
    pub opinion: Opinion,
}

impl Packable for Conflict {
    type Error = Error;
    
    fn packed_len(&self) -> usize {
        TRANSACTION_ID_LENGTH + 0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.opinion.pack(writer)?; 

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let transaction_id = TransactionId::unpack(reader)?;
        let opinion = Opinion::unpack(reader)?;
        
        Ok(Self {
            id: transaction_id,
            opinion,
        })
    }
}

/// Holds a message ID and its timestamp opinion.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timestamp {
    /// Message ID.
    pub id: MessageId,
    /// Opinion of the message timestamp.
    pub opinion: Opinion,
}

impl Packable for Timestamp {
    type Error = Error;
    
    fn packed_len(&self) -> usize {
        MESSAGE_ID_LENGTH + 0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.opinion.pack(writer)?; 

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let message_id = MessageId::unpack(reader)?;
        let opinion = Opinion::unpack(reader)?;
        
        Ok(Self {
            id: message_id,
            opinion,
        })
    }
}