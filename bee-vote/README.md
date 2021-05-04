# bee-vote

FPC voting components for Bee. For now, this is a relatively straightforward port of `goshimmer`'s FPC 
[package](https://github.com/iotaledger/goshimmer/tree/develop/packages/vote), minus the networking
and GRPC functionality. 

Instead, this crate contains a library of all components needed to build FPC voting functionality.

## Example
The following snippet creates a voter, and performs 5 voting rounds on a `Conflict` voting object:

```rust
// Imports
use bee_vote::{Event, FpcBuilder, OpinionGiver}; 

// Create a channel to send voting events through.
let (tx, rx) = flume::unbounded();

// Create a voter (an instance of the `fpc::Fpc` struct)
// Here, the variable `f` is a closure that generates a `Vec` of `OpinionGiver` trait objects:
//  - `Fn() -> Vec<Box<dyn OpinionGiver>>`
let voter = FpcBuilder::default()
    .with_tx(tx)
    .with_opinion_giver_fn(f)
    .build()
    .unwrap();

// Queue a vote, using an ID, `ObjectType`, and an initial opinion of this voter.
block_on(voter.vote("example".to_string(), ObjectType::Conflict, Opinion::Like)).unwrap();

// Perform 5 voting rounds with an opinion threshold of 0.5
for _ in 0..5 {
    block_on(voter.do_round(0.5)).unwrap();
}

let iter = rx.try_iter();
while let Some(ev) = iter.next() {
    // Do something with the received event 
} 
```