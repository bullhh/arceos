use core::error::Error;

extern crate alloc;

use arm_gic_driver::v3::Gic;
use somehal::{
    driver::{
        intc::{Box, Vec},
        probe::{HardwareKind, ProbeDevInfo},
        register::Node,
        *,
    },
    module_driver,
};

use crate::mem::iomap;

module_driver!(
    name: "GICv3",
    kind: DriverKind::Intc,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,gic-v3"],
            on_probe: probe_gic
        }
    ]
);

fn probe_gic(node: Node<'_>, _dev: ProbeDevInfo) -> Result<Vec<HardwareKind>, Box<dyn Error>> {
    let mut reg = node
        .reg()
        .ok_or(alloc::format!("[{}] has no reg", node.name()))?;

    let gicd_reg = reg.next().unwrap();
    let gicc_reg = reg.next().unwrap();

    let gicd = iomap(
        (gicd_reg.address as usize).into(),
        gicd_reg.size.unwrap_or(0x1000),
    )?;
    let gicc = iomap(
        (gicc_reg.address as usize).into(),
        gicc_reg.size.unwrap_or(0x1000),
    )?;

    Ok(alloc::vec![HardwareKind::Intc(Box::new(Gic::new(
        gicd,
        gicc,
        Default::default()
    )))])
}
