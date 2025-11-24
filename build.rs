// Build script to configure platform-specific settings
//
// This script sets the stack size for Windows builds to 8MB to match Unix defaults.
// Windows defaults to 1MB stack size which is insufficient for deep call stacks
// in Git operations performed by integration tests.

fn main() {
    // Set stack size for Windows platforms
    // This affects both the main binary and test binaries
    if cfg!(target_os = "windows") {
        if cfg!(target_env = "msvc") {
            // MSVC linker syntax
            println!("cargo:rustc-link-arg=/STACK:8388608");
        } else if cfg!(target_env = "gnu") {
            // MinGW/GNU linker syntax
            println!("cargo:rustc-link-arg=-Wl,--stack,8388608");
        }
    }

    // Rerun this script if it changes
    println!("cargo:rerun-if-changed=build.rs");
}
