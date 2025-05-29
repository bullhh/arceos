extern crate alloc;

use core::{error::Error, ptr::NonNull};

use alloc::{boxed::Box, format};
use axhal::{
    mem::{MemoryAddr, PhysAddr, phys_to_virt},
    paging::MappingFlags,
};
pub use rdrive::dev_list;

#[cfg(feature = "block")]
pub mod block;

pub mod clk;

#[cfg(feature = "intc")]
pub mod intc;

/// maps a mmio physical address to a virtual address.
fn iomap(addr: PhysAddr, size: usize) -> Result<NonNull<u8>, Box<dyn Error>> {
    let end = (addr.as_usize() + size).align_up_4k();
    let start = addr.align_down_4k();
    let size = end - start.as_usize();

    let start_virt = phys_to_virt(start);

    axmm::kernel_aspace()
        .lock()
        .map_linear(
            start_virt,
            addr,
            size,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::DEVICE,
        )
        .map_err(|e| format!("{e:?}"))?;

    Ok(NonNull::new(start_virt.as_mut_ptr()).unwrap())
}
