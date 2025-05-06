mod gic;
mod timer;

#[cfg(feature = "irq")]
pub use gic::*;
