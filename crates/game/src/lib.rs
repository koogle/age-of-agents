//! The authoritative, deterministic Age of Agents simulation. The server runs
//! it for the hosted world; the native client runs the same code in-process.
mod game;
pub mod navigation;

pub use game::*;
