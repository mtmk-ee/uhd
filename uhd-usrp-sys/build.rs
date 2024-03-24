use std::{
    env,
    path::{Path, PathBuf},
};

use bindgen::EnumVariation;

pub fn main() {
    for path in link_search_dirs() {
        println!("cargo:rustc-link-search={}", path.to_str().unwrap());
    }

    if cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=static=uhd");
    } else {
        println!("cargo:rustc-link-lib=dylib=uhd");
    }

    let bindings_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    write_bindings(&bindings_path);
}

pub fn write_bindings(path: &Path) {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_item("uhd_.+")
        .default_enum_style(EnumVariation::ModuleConsts)
        .derive_default(true)
        .generate()
        .expect("failed to generate bindings");

    bindings
        .write_to_file(path)
        .expect("failed to write bindings.rs");
}

fn link_search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];
    match target_os().as_deref() {
        Some("linux") => dirs.extend([PathBuf::from("/usr/local/lib"), PathBuf::from("/usr/lib")]),
        Some(n) => panic!("unsupported os: {n}"),
        _ => panic!("unsupported os"),
    };
    dirs
}

fn target_os() -> Option<String> {
    env::var("CARGO_CFG_TARGET_OS")
        .ok()
        .map(|s| s.to_lowercase())
}
