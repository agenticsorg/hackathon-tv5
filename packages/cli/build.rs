// Build script for ARW CLI
// Handles NAPI-RS build configuration when the napi feature is enabled

fn main() {
    #[cfg(feature = "napi")]
    {
        // Configure NAPI-RS build
        napi_build::setup();
    }

    // For non-NAPI builds, no special configuration needed
    println!("cargo:rerun-if-changed=build.rs");
}
