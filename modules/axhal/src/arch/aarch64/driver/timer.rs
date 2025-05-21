use core::error::Error;

use aarch64_cpu::registers::*;
use alloc::boxed::Box;

use somehal::{
    driver::{Descriptor, DriverGeneric, HardwareKind, intc::IrqConfig, register::*, systick::*},
    module_driver,
};

module_driver!(
    name: "ARMv8 Timer",
    level: ProbeLevel::PreKernel,
    priority: ProbePriority::DEFAULT,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,armv8-timer"],
            on_probe: probe_timer
        }
    ]
);

#[derive(Clone)]
struct ArmV8Timer {
    irq: IrqConfig,
}

impl Interface for ArmV8Timer {
    fn get_current_cpu(&mut self) -> Box<dyn InterfaceCPU> {
        Box::new(self.clone())
    }
}

impl InterfaceCPU for ArmV8Timer {
    fn set_timeval(&self, ticks: u64) {
        #[cfg(not(feature = "hv"))]
        CNTP_TVAL_EL0.set(ticks);
        #[cfg(feature = "hv")]
        unsafe {
            core::arch::asm!("msr CNTHP_TVAL_EL2, {0:x}", in(reg) ticks)
        };
    }

    fn current_ticks(&self) -> u64 {
        CNTPCT_EL0.get()
    }

    fn tick_hz(&self) -> u64 {
        CNTFRQ_EL0.get()
    }

    #[cfg(feature = "hv")]
    fn set_irq_enable(&self, enable: bool) {
        CNTHP_CTL_EL2.modify(if enable {
            CNTHP_CTL_EL2::ISTATUS::SET + CNTHP_CTL_EL2::IMASK::CLEAR
        } else {
            CNTHP_CTL_EL2::ISTATUS::CLEAR + CNTHP_CTL_EL2::IMASK::SET
        });
    }

    #[cfg(not(feature = "hv"))]
    fn set_irq_enable(&self, enable: bool) {
        CNTP_CTL_EL0.modify(if enable {
            CNTP_CTL_EL0::IMASK::CLEAR
        } else {
            CNTP_CTL_EL0::IMASK::SET
        });
    }

    #[cfg(not(feature = "hv"))]
    fn get_irq_status(&self) -> bool {
        CNTP_CTL_EL0.is_set(CNTP_CTL_EL0::ISTATUS)
    }

    #[cfg(feature = "hv")]
    fn get_irq_status(&self) -> bool {
        CNTHP_CTL_EL2.is_set(CNTHP_CTL_EL2::ISTATUS)
    }

    fn irq(&self) -> IrqConfig {
        self.irq.clone()
    }
}

impl DriverGeneric for ArmV8Timer {
    fn open(&mut self) -> Result<(), ErrorBase> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), ErrorBase> {
        Ok(())
    }
}

fn probe_timer(_node: FdtInfo<'_>, desc: &Descriptor) -> Result<HardwareKind, Box<dyn Error>> {
    #[cfg(not(feature = "hv"))]
    let irq_idx = 1;
    #[cfg(feature = "hv")]
    let irq_idx = 3;

    Ok(HardwareKind::Systick(Box::new(ArmV8Timer {
        irq: desc.irqs[irq_idx].clone(),
    })))
}
