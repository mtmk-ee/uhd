use std::env;
use std::path::PathBuf;

use bindgen::EnumVariation;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=uhd");


    let bindings = bindgen::Builder::default()
        .header("/usr/local/include/uhd.h")
        .default_enum_style(EnumVariation::ModuleConsts)
        .allowlist_function("^uhd_.+")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
