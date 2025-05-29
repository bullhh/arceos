extern crate alloc;

use axhal::time::{busy_wait_until, wall_time};

use alloc::boxed::Box;
use core::error::Error;
use core::time::Duration;

use rk3568_driver_block::{EMmcHost, EmmcDriver, Kernel, init_clk, set_impl};

use rdrive::{Descriptor, HardwareKind, module_driver, register::FdtInfo};

use crate::drivers_dyn::iomap;

pub struct KernelImpl;

impl Kernel for KernelImpl {
    fn sleep(us: u64) {
        let current_time = wall_time();
        let duration = Duration::from_micros(us);
        let deadline = current_time + duration;
        busy_wait_until(deadline);
    }
}

set_impl!(KernelImpl);

module_driver!(
    name: "Rockchip SDHCI",
    level: ProbeLevel::PostKernel,
    priority: ProbePriority::DEFAULT,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["rockchip,dwcmshc-sdhci"],
            on_probe: probe_mmc
        }
    ],
);

fn probe_mmc(info: FdtInfo<'_>, _dev: &Descriptor) -> Result<HardwareKind, Box<dyn Error>> {
    let mut reg = info
        .node
        .reg()
        .ok_or(alloc::format!("[{}] has no reg", info.node.name()))?;

    let mmc_reg = reg.next().unwrap();

    let mmc: core::ptr::NonNull<u8> = iomap(
        (mmc_reg.address as usize).into(),
        mmc_reg.size.unwrap_or(0x10000),
    )?;

    let _ = init_clk(0x7c);

    let mmc_address = mmc.as_ptr() as usize;

    debug!("mmc address: {:#x}", mmc_address);
    let mut emmc = EMmcHost::new(mmc_address);

    if emmc.init().is_ok() {
        log::info!("RK3568 eMMC: successfully initialized");
    } else {
        log::warn!("RK3568 eMMC: init failed");
    }

    Ok(HardwareKind::Block(Box::new(EmmcDriver(emmc))))
}
