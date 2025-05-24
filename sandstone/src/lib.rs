#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]
#![allow(dead_code)]

//! # Sandstone
//! A Minecraft: Java Edition protocol library.
//!
//! See the project on GitHub [dec4234/sandstone](https://www.github.com/dec4234/sandstone)
//! or crates.io [sandstone](https://crates.io/crates/sandstone) for more info.

pub mod protocol_types;
pub mod util;
pub mod protocol;
pub mod network;
pub mod game;