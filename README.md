Orderbook

A limit order book matching engine written in Rust

What this is

A price time priority matching engine:incoming limit orders are matched against resting orders
at the best available price, with FIFO ordering within each price level.It implements the core
mechanics you'd find in a real exchange's matching core, order insertion,cancellation and
crossing/matching logic. 

Limitations

This is a matching engine core, not a production level exchange.Alot of things have been
deliberately left out of scope. Some processes like cancellation could be improved by
implementing a better data structure.

Design

types.rs - newtype wrappers around raw primitives to make sure invalid states are compile time
errors rather than a runtime bug

orderbook.rs - the book itself with insert and cancel operations, it also has the matching engine
logic.
