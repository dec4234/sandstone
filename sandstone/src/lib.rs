#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]
#![allow(dead_code)]
#![allow(elided_lifetimes_in_paths)]
#![allow(mismatched_lifetime_syntaxes)]

//! # Sandstone
//! A Minecraft: Java Edition protocol library.
//!
//! See the project on GitHub [dec4234/sandstone](https://www.github.com/dec4234/sandstone)
//! or crates.io [sandstone](https://crates.io/crates/sandstone) for more info.

pub mod game;
pub mod network;
pub mod protocol;
pub mod protocol_types;
pub mod util;
