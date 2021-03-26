// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! FPC voting components for Bee. For now, this is a relatively straightforward port of `goshimmer`'s FPC 
//! [package](https://github.com/iotaledger/goshimmer/tree/develop/packages/vote), minus the networking
//! and GRPC functionality. 
//!
//! Instead, this crate contains a library of all components needed to build FPC voting functionality.

mod context;
pub mod error;
pub mod events;
pub mod fpc;
pub mod opinion;

pub use error::Error;
pub use events::Event;
pub use fpc::{Fpc, FpcBuilder};
pub use opinion::{Opinion, OpinionGiver};
