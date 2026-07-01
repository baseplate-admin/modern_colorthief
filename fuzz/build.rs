fn main() {
    // Required for cargo-fuzz compatibility.
    // Mirrors wasmtime's fuzz/build.rs pattern.
    println!("cargo:rerun-if-changed=build.rs");
}
