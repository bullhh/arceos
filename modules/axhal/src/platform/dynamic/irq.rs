pub use crate::arch::dispatch_irq;
use somehal::driver::{DeviceId, intc::HardwareCPU};

static mut IRQ_CHIP: u64 = 0;

pub(crate) unsafe fn init() {
    let ls = somehal::driver::read(|m| m.intc.all());
    let (id, chip) = ls.first().unwrap();

    unsafe { IRQ_CHIP = (*id).into() };
}

pub(crate) fn cpu_interface() -> &'static HardwareCPU {
    somehal::irq::interface(unsafe { IRQ_CHIP }.into()).expect("no cpu interface")
}
