#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]
#![allow(dead_code)]
#![allow(elided_lifetimes_in_paths)]
#![allow(mismatched_lifetime_syntaxes)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Sandstone
//! A Minecraft: Java Edition protocol library.
//!
//! See the project on GitHub [dec4234/sandstone](https://www.github.com/dec4234/sandstone)
//! or crates.io [sandstone](https://crates.io/crates/sandstone) for more info.
//!
//! ## Feature flags
#![doc = document_features::document_features!()]

pub mod game;
pub mod network;
pub mod protocol;
pub mod protocol_types;
pub mod util;

/// The Mojang API crate (`sandstone-mojang-api`), for types such as [mojang_api::HasJoinedResponse].
pub use mojang_api;
