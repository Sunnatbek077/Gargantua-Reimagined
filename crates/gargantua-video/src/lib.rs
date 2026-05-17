// FILE: crates/gargantua-video/src/lib.rs
// See header comments in sub-modules for full specifications.

pub mod color;
pub mod config;
pub mod denoise;
pub mod encode;
pub mod errors;
pub mod offline;
pub mod realtime;

pub use errors::VideoError;
