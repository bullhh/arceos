fn main() {
    println!("cargo::rustc-check-cfg=cfg(plat_dyn)");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if std::env::var("CARGO_FEATURE_PLAT_DYN").is_ok() && arch.contains("aarch") {
        println!("cargo:rustc-cfg=plat_dyn");
    }
}
