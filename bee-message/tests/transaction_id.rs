// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::str::FromStr;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const TRANSACTION_ID_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const TRANSACTION_ID_INVALID_LEN: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";

#[test]
fn from_str_valid() {
    TransactionId::from_str(TRANSACTION_ID).unwrap();
}

#[test]
fn from_str_invalid_hex() {
    assert!(matches!(
        TransactionId::from_str(TRANSACTION_ID_INVALID_HEX),
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == TRANSACTION_ID_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len() {
    assert!(matches!(
        TransactionId::from_str(TRANSACTION_ID_INVALID_LEN),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(
        TRANSACTION_ID,
        TransactionId::from_str(TRANSACTION_ID).unwrap().to_string()
    );
}

#[test]
fn packed_len() {
    assert_eq!(TransactionId::from_str(TRANSACTION_ID).unwrap().packed_len(), 32);
}
