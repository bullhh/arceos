use core::error::Error;

extern crate alloc;

use arm_gic_driver::v3::Gic;
use somehal::{
    driver::{
        Descriptor, HardwareKind,
        intc::{Box, DriverGeneric, Interface, Vec},
        register::Node,
    },
    mem::cpu_idx,
    module_driver,
};

use crate::mem::iomap;

use super::Reg;

module_driver!(
    name: "GICv3",
   level: ProbeLevel::PreKernel,
    priority: ProbePriority::INTC,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,gic-v3"],
            on_probe: probe_gic
        }
    ]
);

struct GicV3 {
    inner: Gic,
    gicd: Reg,
    gicr: Reg,
}

impl DriverGeneric for GicV3 {
    fn open(&mut self) -> arm_gic_driver::DriverResult {
        self.inner.open()
    }

    fn close(&mut self) -> arm_gic_driver::DriverResult {
        self.inner.close()
    }
}

impl Interface for GicV3 {
    fn cpu_interface(&self) -> arm_gic_driver::HardwareCPU {
        if !cpu_idx().is_primary() {
            let _ = iomap(self.gicr.addr.into(), self.gicr.size);
            let _ = iomap(self.gicd.addr.into(), self.gicd.size);
        }

        self.inner.cpu_interface()
    }

    fn irq_enable(&mut self, irq: arm_gic_driver::IrqId) {
        self.inner.irq_enable(irq);
    }

    fn irq_disable(&mut self, irq: arm_gic_driver::IrqId) {
        self.inner.irq_disable(irq);
    }

    fn set_priority(&mut self, irq: arm_gic_driver::IrqId, priority: usize) {
        self.inner.set_priority(irq, priority);
    }

    fn set_trigger(&mut self, irq: arm_gic_driver::IrqId, trigger: arm_gic_driver::Trigger) {
        self.inner.set_trigger(irq, trigger);
    }

    fn set_target_cpu(&mut self, irq: arm_gic_driver::IrqId, cpu: arm_gic_driver::CpuId) {
        self.inner.set_target_cpu(irq, cpu);
    }

    fn capabilities(&self) -> Vec<arm_gic_driver::Capability> {
        self.inner.capabilities()
    }
}

fn probe_gic(node: Node<'_>, _dev: &Descriptor) -> Result<HardwareKind, Box<dyn Error>> {
    let mut reg = node
        .reg()
        .ok_or(alloc::format!("[{}] has no reg", node.name()))?;

    let gicd_reg = reg.next().unwrap();
    let gicr_reg = reg.next().unwrap();

    let gicd = iomap(
        (gicd_reg.address as usize).into(),
        gicd_reg.size.unwrap_or(0x1000),
    )?;
    let gicr = iomap(
        (gicr_reg.address as usize).into(),
        gicr_reg.size.unwrap_or(0x1000),
    )?;

    let gic = GicV3 {
        inner: Gic::new(gicd, gicr, Default::default()),
        gicr: Reg {
            addr: gicr_reg.address as _,
            size: gicr_reg.size.unwrap_or(0x1000),
        },
        gicd: Reg {
            addr: gicd_reg.address as _,
            size: gicd_reg.size.unwrap_or(0x1000),
        },
    };

    Ok(HardwareKind::Intc(Box::new(gic)))
}
