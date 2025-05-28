use std::io::Result;
use std::path::Path;

const BUILTIN_PLATFORMS: &[&str] = &[
    "aarch64-bsta1000b",
    "aarch64-qemu-virt",
    "aarch64-raspi4",
    "loongarch64-qemu-virt",
    "aarch64-rk3588j",
    "riscv64-qemu-virt",
    "x86_64-pc-oslab",
    "x86_64-qemu-q35",
];

const BUILTIN_PLATFORM_FAMILIES: &[&str] = &[
    "aarch64-bsta1000b",
    "aarch64-phytium-pi",
    "aarch64-qemu-virt",
    "aarch64-raspi",
    "loongarch64-qemu-virt",
    "aarch64-rk3588j",
    "riscv64-qemu-virt",
    "x86-pc",
];

const DYN_PLATFORMS: &[&str] = &["aarch64-dyn"];

fn make_cfg_values(str_list: &[&str]) -> String {
    str_list
        .iter()
        .map(|s| format!("{:?}", s))
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let platform = axconfig::PLATFORM;
    if platform != "dummy" {
        gen_linker_script(&arch, platform).unwrap();
    }

    println!("cargo:rustc-cfg=platform=\"{}\"", platform);
    println!(
        "cargo:rustc-cfg=platform_family=\"{}\"",
        axconfig::plat::FAMILY
    );
    println!(
        "cargo::rustc-check-cfg=cfg(platform, values({}))",
        make_cfg_values(BUILTIN_PLATFORMS)
    );
    println!(
        "cargo::rustc-check-cfg=cfg(platform_family, values({}))",
        make_cfg_values(BUILTIN_PLATFORM_FAMILIES)
    );
}

fn gen_linker_script(arch: &str, platform: &str) -> Result<()> {
    let fname = format!("linker_{}.lds", platform);
    let is_dyn = DYN_PLATFORMS.contains(&platform);

    println!("cargo::rustc-check-cfg=cfg(plat_dyn)");

    if is_dyn {
        println!("cargo:rustc-cfg=plat_dyn");
    }

    let output_arch = if arch == "x86_64" {
        "i386:x86-64"
    } else if arch.contains("riscv") {
        "riscv" // OUTPUT_ARCH of both riscv32/riscv64 is "riscv"
    } else {
        arch
    };

    let ld_template_name = if is_dyn {
        "linker.dyn.lds.S"
    } else {
        "linker.lds.S"
    };

    let ld_content = std::fs::read_to_string(ld_template_name)?;
    let ld_content = ld_content.replace("%ARCH%", output_arch);
    let ld_content = ld_content.replace(
        "%KERNEL_BASE%",
        &format!("{:#x}", axconfig::plat::KERNEL_BASE_VADDR),
    );
    let ld_content = ld_content.replace("%SMP%", &format!("{}", axconfig::SMP));

    // target/<target_triple>/<mode>/build/axhal-xxxx/out
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // target/<target_triple>/<mode>/linker_xxxx.lds
    let out_path = Path::new(&out_dir).join("../../..").join(fname);
    let _ = std::fs::remove_file(&out_path);
    std::fs::write(out_path, ld_content)?;
    Ok(())
}
