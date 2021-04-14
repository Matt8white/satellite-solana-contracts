#![deny(missing_docs)]

//! Sattelite friends solana program

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

/// Current program version
pub const PROGRAM_VERSION: u8 = 1;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

// FIXME: change this address before run tests/program
solana_program::declare_id!("11111111111111111111111111111111");