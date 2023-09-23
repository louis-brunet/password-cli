#![warn(rust_2018_idioms)]
// #![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

pub mod cli;
pub mod encryption;
pub mod model;

#[cfg(test)]
mod bench;

