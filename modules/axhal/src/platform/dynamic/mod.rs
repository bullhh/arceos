use memory_addr::{MemoryAddr, PhysAddr, VirtAddr};
use page_table_entry::MappingFlags;
use somehal::driver::DeviceId;

use crate::mem::{self, MapLinearFunc, phys_to_virt};

#[cfg(feature = "irq")]
pub(crate) mod irq;

unsafe extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

#[somehal::entry]
fn main(cpu_id: usize, dtb: usize) -> ! {
    unsafe { rust_main(cpu_id, dtb) };
}

pub mod console {
    pub fn write_bytes(bytes: &[u8]) {
        somehal::console::write_bytes(bytes);
    }

    pub fn read_bytes(bytes: &mut [u8]) -> usize {
        panic!("read_bytes is not implemented yet");
    }
}

pub mod time {
    pub fn current_ticks() -> u64 {
        somehal::systime::current_ticks()
    }

    /// Converts hardware ticks to nanoseconds.
    #[inline]
    pub fn ticks_to_nanos(ticks: u64) -> u64 {
        somehal::systime::ticks_to_nanos(ticks) as _
    }

    /// Converts nanoseconds to hardware ticks.
    #[inline]
    pub fn nanos_to_ticks(nanos: u64) -> u64 {
        somehal::systime::nanos_to_ticks(nanos as _)
    }

    /// Return epoch offset in nanoseconds (wall time offset to monotonic clock start).
    pub fn epochoffset_nanos() -> u64 {
        0
    }
}
pub mod misc {
    pub fn terminate() -> ! {
        somehal::power::terminate()
    }
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init(map_func: MapLinearFunc) {
    unsafe {
        mem::init_map_liner(map_func);
        somehal::init();
        #[cfg(feature = "irq")]
        irq::init();
    }
}
