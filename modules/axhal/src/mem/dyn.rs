use memory_addr::{PhysAddr, VirtAddr};
use somehal::mem::region::{AccessFlags, MemRegionKind};

use super::{MemRegion, MemRegionFlags};

/// Converts a virtual address to a physical address.
#[inline]
pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    let paddr = somehal::mem::virt_to_phys(vaddr.as_usize().into());
    paddr.raw().into()
}

/// Converts a physical address to a virtual address.
#[inline]
pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    let vaddr = somehal::mem::phys_to_virt(paddr.as_usize().into());
    vaddr.raw().into()
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    somehal::mem::memory_regions().map(|reg| reg.into())
}

impl From<somehal::mem::MemRegion> for MemRegion {
    fn from(value: somehal::mem::MemRegion) -> Self {
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
