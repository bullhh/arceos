extern crate alloc;

use crate::structs::iomap;

use core::error::Error;

use rk3568_driver_clk::ClkDriver;

use rdrive::{Descriptor, HardwareKind, clk::*, module_driver, register::FdtInfo};

module_driver!(
    name: "rk3568 CRU",
    level: ProbeLevel::PreKernel,
    priority: ProbePriority::CLK,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["rockchip,rk3568-cru"],
            on_probe: probe_clk
        }
    ],
);

fn probe_clk(info: FdtInfo<'_>, _dev: &Descriptor) -> Result<HardwareKind, Box<dyn Error>> {
    let mut reg = info
        .node
        .reg()
        .ok_or(alloc::format!("[{}] has no reg", info.node.name()))?;

    let cru_reg = reg.next().unwrap();

    let cru: core::ptr::NonNull<u8> = iomap(
        (cru_reg.address as usize).into(),
        cru_reg.size.unwrap_or(0x1000),
    )?;

    let cru_address = cru.as_ptr() as u64;

    debug!("cru address: {:#x}", cru_address);

    Ok(HardwareKind::Clk(Box::new(ClkDriver::new(cru_address))))
}
