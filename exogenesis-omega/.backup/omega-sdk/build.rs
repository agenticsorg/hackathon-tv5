// Build script for omega-sdk
//
// This script runs during cargo build and can:
// - Generate C header files with cbindgen
// - Copy headers to include/ directory
// - Set up platform-specific build flags

use std::env;
use std::path::PathBuf;

fn main() {
    // Get output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=include/");

    // TODO: Enable cbindgen for automatic C header generation
    // This is commented out because cbindgen is not yet a dependency
    // Uncomment when cbindgen is added to build-dependencies

    /*
    generate_c_header(&out_dir);
    */

    // For now, we manually maintain include/omega_sdk.h
    // When cbindgen is enabled, it will auto-generate the header

    println!("cargo:warning=Using manually maintained C header at include/omega_sdk.h");
    println!("cargo:warning=TODO: Enable cbindgen for automatic header generation");
}

// TODO: Uncomment this function when cbindgen is added as a build dependency
/*
fn generate_c_header(out_dir: &PathBuf) {
    use std::env;

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Configure cbindgen
    let config = cbindgen::Config {
        language: cbindgen::Language::C,
        cpp_compat: true,
        include_guard: Some("OMEGA_SDK_H".to_string()),
        documentation: true,
        style: cbindgen::Style::Both,
        ..Default::default()
    };

    // Generate header
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(out_dir.join("omega_sdk_generated.h"));

    println!("cargo:warning=Generated C header at {:?}", out_dir.join("omega_sdk_generated.h"));
}
*/

// Platform-specific configuration
#[allow(dead_code)]
fn configure_platform() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    match (target_os.as_str(), target_arch.as_str()) {
        ("linux", "aarch64") => {
            // ARM64 Linux (typical TV platform)
            println!("cargo:rustc-link-arg=-Wl,--as-needed");
            println!("cargo:warning=Building for ARM64 Linux (TV platform)");
        }
        ("linux", "x86_64") => {
            // x86_64 Linux (development/testing)
            println!("cargo:warning=Building for x86_64 Linux");
        }
        _ => {
            println!("cargo:warning=Building for unknown platform: {} {}", target_os, target_arch);
        }
    }
}

// Instructions for integrating cbindgen:
//
// 1. Add to Cargo.toml build-dependencies:
//    cbindgen = "0.26"
//
// 2. Uncomment the generate_c_header function above
//
// 3. Call generate_c_header in main():
//    generate_c_header(&out_dir);
//
// 4. The C header will be auto-generated at target/<profile>/build/omega-sdk-*/out/omega_sdk_generated.h
//
// 5. Consider creating a cbindgen.toml configuration file for fine-grained control
