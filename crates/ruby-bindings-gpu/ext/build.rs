use std::process::Command;

fn main() {
    // Query Ruby for its library path and link flags.
    // This ensures libruby is linked on all platforms, especially macOS
    // where rb-sys-env may not output link flags reliably.
    let output = Command::new("ruby")
        .args(["-r", "mkmf", "-e", "puts Config::CONFIG['libdir']"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    if let Some(libdir) = output {
        let libdir = libdir.trim();
        if !libdir.is_empty() {
            println!("cargo:rustc-link-search={}", libdir);
        }
    }

    println!("cargo:rustc-link-lib=ruby");
}
