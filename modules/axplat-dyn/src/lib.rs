//! A dyn platform for axhal.

#![no_std]

extern crate alloc;

#[cfg(target_arch = "aarch64")]
pub use somehal::*;
