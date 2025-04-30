#[cfg(feature = "irq")]
mod gic;
mod power;
mod timer;

#[cfg(feature = "irq")]
pub use gic::*;
