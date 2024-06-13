//! This module defines the network protocol for the server and client.
//! 
//! This includes data types, serializers, packet implementations and client & server handlers.
//! 
//! See the documentation for the `client` and `server` modules for more information on how to use the network API. 

pub mod network_error;
pub mod client;
pub mod server;