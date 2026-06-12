use std::process::Command;

fn main() {
    // Ensure libruby is linked on all platforms.
    // magnus 0.8's rb-sys-env does not emit link flags on macOS,
    // so we query Ruby directly and pass the flags to the linker.
    let output = Command::new("ruby")
        .args(["-r", "mkmf", "-e", "puts Config::CONFIG['libdir']"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    if let Some(libdir) = output {
        let libdir = libdir.trim().to_string();
        if !libdir.is_empty() {
            println!("cargo:rustc-link-arg=-L{}", libdir);
        }
    }

    println!("cargo:rustc-link-arg=-lruby");
}
