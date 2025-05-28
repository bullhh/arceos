mod v2;
mod v3;

#[cfg(feature = "irq")]
/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(irq_no: usize) {
    let icc = crate::platform::irq::cpu_interface();
    let intid = if irq_no == 0 {
        match icc.ack() {
            Some(v) => v,
            None => return,
        }
    } else {
        axplat_dyn::driver::IrqId::from(irq_no)
    };
    crate::irq::dispatch_irq_common(intid.into());
    icc.eoi(intid);
    if icc.get_eoi_mode() {
        icc.dir(intid);
    }
}

#[cfg(feature = "irq")]
pub fn fetch_irq() -> usize {
    let icc = crate::platform::irq::cpu_interface();
    icc.ack().map(|o| o.into()).unwrap_or_default()
}
