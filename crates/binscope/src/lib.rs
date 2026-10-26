//! Read-only static inspection of executables and libraries.
//!
//! `binscope` answers "what is this file and what changed?" for builds you
//! ship or depend on: format, architecture, sections, entropy, linked
//! libraries, imported and exported symbols, embedded strings and a diff
//! between two versions. It never executes, patches or unpacks anything.

pub mod access;
pub mod diff;
pub mod entropy;
pub mod inspect;
pub mod strings;
