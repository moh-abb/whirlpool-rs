use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=/usr/lib/include");

    // Will need to run `make src/patches.h` in order to generate the Juno
    // patches etc.
    // Note also that pyamy.c is not compiled in.
    cc::Build::new()
        .file("amy/src/algorithms.c")
        .file("amy/src/amy.c")
        .file("amy/src/delay.c")
        .file("amy/src/envelope.c")
        .file("amy/src/filters.c")
        .file("amy/src/parse.c")
        .file("amy/src/sequencer.c")
        .file("amy/src/transfer.c")
        .file("amy/src/midi_mappings.c")
        .file("amy/src/custom.c")
        .file("amy/src/patches.c")
        .file("amy/src/libminiaudio-audio.c")
        .file("amy/src/oscillators.c")
        .file("amy/src/interp_partials.c")
        .file("amy/src/pcm.c")
        .file("amy/src/log2_exp2.c")
        .file("amy/src/instrument.c")
        .file("amy/src/amy_midi.c")
        .file("amy/src/api.c")
        .file("amy/src/cv_trigger.c")
        .include("amy/src/amy.h")
        .warnings(false) // Suppress C "unused parameter" warnings.
        .compile("amy_lib");

    println!("cargo:rerun-if-changed=amy/src/amy.h");

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
