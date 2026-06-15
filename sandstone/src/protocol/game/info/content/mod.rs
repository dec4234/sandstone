//! Block & item data, sourced at compile time from the PrismarineJS minecraft-data repo.
//!
//! This is a thin re-export of the standalone [`mc_data`] crate (located in
//! `content/mc-data/`); see its docs for details. Usage:
//!
//! ```ignore
//! use sandstone::protocol::game::info::content::Block;
//!
//! let id = Block::Bedrock::get_id();
//! let info = Block::Bedrock::info(); // full BlockInfo metadata
//! ```

pub use mc_data::*;
