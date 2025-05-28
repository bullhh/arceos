//! [ArceOS] hardware abstraction layer, provides unified APIs for
//! platform-specific operations.
//!
//! It does the bootstrapping and initialization process for the specified
//! platform, and provides useful operations on the hardware.
//!
//! Currently supported platforms (specify by cargo features):
//!
//! - `x86-pc`: Standard PC with x86_64 ISA.
//! - `riscv64-qemu-virt`: QEMU virt machine with RISC-V ISA.
//! - `aarch64-qemu-virt`: QEMU virt machine with AArch64 ISA.
//! - `aarch64-raspi`: Raspberry Pi with AArch64 ISA.
//! - `dummy`: If none of the above platform is selected, the dummy platform
//!    will be used. In this platform, most of the operations are no-op or
//!    `unimplemented!()`. This platform is mainly used for [cargo test].
//!
//! # Cargo Features
//!
//! - `smp`: Enable SMP (symmetric multiprocessing) support.
//! - `fp_simd`: Enable floating-point and SIMD support.
//! - `paging`: Enable page table manipulation.
//! - `irq`: Enable interrupt handling support.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos
//! [cargo test]: https://doc.rust-lang.org/cargo/guide/tests.html

#![no_std]
#![feature(naked_functions)]
#![feature(doc_auto_cfg)]
#![feature(sync_unsafe_cell)]
#![cfg_attr(plat_dyn, feature(used_with_arg))]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate memory_addr;

#[cfg(feature = "alloc")]
extern crate alloc;

mod platform;

#[macro_use]
pub mod trap;

pub mod arch;
pub mod cpu;
pub mod mem;
pub mod time;

#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "irq")]
pub mod irq;

#[cfg(feature = "paging")]
pub mod paging;

/// Console input and output.
pub mod console {
    pub use super::platform::console::*;
}

/// Miscellaneous operation, e.g. terminate the system.
pub mod misc {
    pub use super::platform::misc::*;
}

/// Multi-core operations.
#[cfg(feature = "smp")]
pub mod mp;

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
#[allow(unused)]
pub fn platform_init(map_func: Option<AddrMapFunc>) {
    #[cfg(not(plat_dyn))]
    platform::platform_init();
    #[cfg(plat_dyn)]
    platform::platform_init(map_func.unwrap());
}

#[cfg(feature = "smp")]
pub use self::platform::platform_init_secondary;

#[cfg(plat_dyn)]
pub use self::platform::driver;

pub use axerrno::AxError;
use mem::AddrMapFunc;
