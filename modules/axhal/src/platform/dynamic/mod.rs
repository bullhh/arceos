pub use somehal::driver;
pub use somehal::driver::intc::IrqConfig;

use crate::mem::{self, MapLinearFunc};

#[cfg(feature = "irq")]
pub(crate) mod irq;

#[cfg(feature = "smp")]
pub(crate) mod mp;

unsafe extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

#[somehal::entry]
fn main(cpu_id: usize, cpu_idx: usize) -> ! {
    if cpu_idx == 0 {
        crate::cpu::init_primary(cpu_id);
        unsafe { rust_main(cpu_id, 0) };
    } else {
        crate::cpu::init_secondary(cpu_id);
        #[cfg(feature = "smp")]
        unsafe {
            rust_main_secondary(cpu_id)
        }
    }
}

pub mod console {
    pub fn write_bytes(bytes: &[u8]) {
        somehal::console::write_bytes(bytes);
    }

    pub fn read_bytes(_bytes: &mut [u8]) -> usize {
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

    /// Set a one-shot timer.
    ///
    /// A timer interrupt will be triggered at the given deadline (in nanoseconds).
    pub fn set_oneshot_timer(deadline_ns: u64) {
        let ticks = current_ticks();
        let deadline = nanos_to_ticks(deadline_ns);
        let interval = if ticks < deadline {
            let interval = deadline - ticks;
            debug_assert!(interval <= u32::MAX as u64);
            interval
        } else {
            0
        };

        somehal::systime::get().set_timeval(interval);
        somehal::systime::get().set_irq_enable(true);
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

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    unsafe {
        #[cfg(feature = "irq")]
        irq::init_secondary();
    }
}
