fn main() -> miette::Result<()> {
    // Cargo envs
    let pkgname = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME was not set");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR was not set");
    println!(
        "==== build.rs: CARGO_PKG_NAME={} CARGO_MANIFEST_DIR={} ====",
        &pkgname, &manifest_dir
    );

    let linkage = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or(String::new());
    if linkage.contains("crt-static") {
        println!("==== build.rs: the C runtime will be statically linked ====");
    } else {
        println!("==== build.rs: the C runtime will be dynamically linked ====");
    }

    // Other envs
    let profile = std::env::var("PROFILE").expect("PROFILE was not set");
    let target = std::env::var("TARGET").expect("TARGET was not set");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR was not set");
    println!(
        "==== build.rs: PROFILE={} TARGET={} OUT_DIR={} ====",
        &profile, &target, &out_dir
    );

    // Bridge -- cxx
    let ffi_files = vec!["ffi/filter.rs"];
    for file in &ffi_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    // Re-run
    println!("cargo:rerun-if-changed=src/lib.rs");

    // Cxx
    cxx_build::bridges(ffi_files)
        .flag("-I/usr/local/include")
        .flag_if_supported("-std=c++17")
        .compile("safecomm_bridge");

    // Add instructions to link to any C++ libraries you need.
    Ok(())
}
