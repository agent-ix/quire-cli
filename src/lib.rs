//! Shared library surface for the `quire` binary.
//!
//! This crate is primarily a binary (`src/main.rs`); the library exists
//! so integration tests can import the path-safety guards and I/O
//! helpers without going through the process boundary.

pub mod io;
pub mod safety;
