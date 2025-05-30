extern crate alloc;

use core::ptr::NonNull;

use axplat_dyn::mem::{
    percpu_data,
    region::{AccessFlags, MemRegionKind},
};
use memory_addr::{MemoryAddr, PhysAddr, VirtAddr};
use page_table_entry::MappingFlags;

use super::{MemRegion, MemRegionFlags};

/// Converts a virtual address to a physical address.
#[inline]
pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    axplat_dyn::mem::virt_to_phys(vaddr.as_usize().into())
        .raw()
        .into()
}

/// Converts a physical address to a virtual address.
#[inline]
pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    axplat_dyn::mem::phys_to_virt(paddr.as_usize().into())
        .raw()
        .into()
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    axplat_dyn::mem::memory_regions().map(|reg| reg.into())
}

impl From<&axplat_dyn::mem::MemRegion> for MemRegion {
    fn from(value: &axplat_dyn::mem::MemRegion) -> Self {
        let mut flags = MemRegionFlags::empty();
        if value.config.access.contains(AccessFlags::Read) {
            flags |= MemRegionFlags::READ;
        }
        if value.config.access.contains(AccessFlags::Write) {
            flags |= MemRegionFlags::WRITE;
        }
        if value.config.access.contains(AccessFlags::Execute) {
            flags |= MemRegionFlags::EXECUTE;
        }
        match value.kind {
            MemRegionKind::Code => {
                flags |= MemRegionFlags::RESERVED;
            }
            MemRegionKind::Stack => {
                flags |= MemRegionFlags::RESERVED;
            }
            MemRegionKind::PerCpu => {
                flags |= MemRegionFlags::RESERVED;
            }
            MemRegionKind::Device => {
                flags |= MemRegionFlags::DEVICE;
            }
            MemRegionKind::Memory => {
                flags |= MemRegionFlags::FREE;
            }
            MemRegionKind::Reserved => {
                flags |= MemRegionFlags::RESERVED;
            }
        }

        Self {
            paddr: value.phys_start.raw().into(),
            size: value.size,
            flags,
            name: value.name,
        }
    }
}

/// Percpu section base address.
///
/// # Safety
///
/// Only used for percpu crate.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _percpu_base() -> *mut u8 {
    unsafe { percpu_data().as_ref().as_ptr() as usize as _ }
}
