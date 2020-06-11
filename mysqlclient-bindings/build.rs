use binding_helpers::{bindings_builder, probe_libs};
use std::env;
use std::path::PathBuf;

fn main() {
    let (_, include_paths) = probe_libs(true);
    build_bindings(include_paths);
}

fn build_bindings(include_paths: Vec<String>) {
    let bindings = bindings_builder(include_paths, true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
