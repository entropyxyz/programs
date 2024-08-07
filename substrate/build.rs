use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("metadata.rs");

    let metadata = fs::read("substrate_metadata.scale").unwrap();
    fs::write(
        dest_path,
        format!("const METADATA: &[u8] = &{:?};\n", metadata),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=substrate_metadata.scale");
}
