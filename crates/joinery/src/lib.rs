//! Woodworking domain logic for the `joinery` MCP server.
//!
//! Everything here is deterministic and offline: the model proposes a design,
//! and this crate does the arithmetic woodworkers get wrong at 11 pm —
//! part sizes, cut lists, sheet yield and whether a shelf will sag.

pub mod assembly;
pub mod cutlist;
pub mod furniture;
pub mod model;
pub mod nesting;
pub mod sag;
pub mod svg;
