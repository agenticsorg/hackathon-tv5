use std::env;
use std::path::PathBuf;

fn main() {
    // Generate C bindings using cbindgen
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("include")
        .join("omega_sdk_generated.h");

    // Note: We have a manually crafted header, but we can generate one too
    // cbindgen::generate(&crate_dir)
    //     .expect("Unable to generate bindings")
    //     .write_to_file(&output_file);

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=include/");
}
