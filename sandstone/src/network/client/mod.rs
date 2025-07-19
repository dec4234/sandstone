//! This file is used to define everything relating to a client connection.
//! This includes the connection itself, the ability to send and receive packets, and the ability to
//! change the packet state of the connection.

use crate::protocol::serialization::{McSerialize, StateBasedDeserializer};
use std::fmt::Display;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub mod client_handlers;

