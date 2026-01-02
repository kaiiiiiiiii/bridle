//! Installation management for bridle.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod discovery;
pub mod installer;
mod types;

pub use discovery::{discover_skills, DiscoveryError};
pub use types::*;
