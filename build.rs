use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PCP_LIB_DIR");
    // For PCP installed outside the default linker search path.
    if let Ok(dir) = env::var("PCP_LIB_DIR") {
        println!("cargo:rustc-link-search=native={dir}");
    }
    println!("cargo:rustc-link-lib=pcp_mmv");
}
