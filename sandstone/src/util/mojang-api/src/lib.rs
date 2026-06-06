//! This file defines the Mojang API - used to get information about users, servers and encryption validation
//! The rate limit is allegedly 600 requests per 10 minutes
//! Reference = https://minecraft.wiki/w/Mojang_API

#![allow(unused)]
#![allow(non_snake_case)]

mod http;
mod profile;
mod uuid;
mod auth;
mod blocked_servers;

pub use auth::*;
pub use blocked_servers::*;
// re-export all the public items in the submodules for easier access
pub use profile::*;
pub use uuid::*;
