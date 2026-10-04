//! The authoritative, deterministic Age of Agents simulation. The server runs
//! it for the hosted world; the native client runs the same code in-process.
//!
//! Core concepts live in [`spatial`], [`units`], [`jobs`] and [`resources`].
//! [`navigation`] provides deterministic grid search. [`GameWorld`] coordinates
//! commands and ticks; the existing root-level type exports remain available.
mod game;
pub mod navigation;

pub use game::*;
