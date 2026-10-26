//! Drive phones from an agent without drowning it in XML.
//!
//! Raw accessibility dumps are thousands of tokens of nested containers.
//! `handset` flattens them into one line per meaningful element, tags each
//! with a short ref (`e12`) the agent can act on, keeps refs stable across
//! snapshots, and answers every action with only what changed.

pub mod android;
pub mod device;
pub mod ios;
pub mod node;
pub mod server;
pub mod session;
pub mod snapshot;
