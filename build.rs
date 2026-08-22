use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=/usr/lib/include");

    println!("cargo:rustc-link-lib=bz2");

    let bindings = bindgen::Builder::default()
        .use_core()
        .header("amy/src/amy.h")
        // Avoid duplicate definitions
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_NAN")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("amy_bindings.rs"))
        .expect("Couldn't write bindings!");
}
