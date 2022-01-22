use std::path::PathBuf;

fn main() {
    // TODO: copy the libvosk.a file to the out dir, to not polute the repo
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // let out_dir: PathBuf = out_dir.into();
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo manifest dir was not set?"));
    println!("cargo:rustc-link-search={}", manifest_dir.display());
    println!("cargo:rustc-link-lib=stdc++");
}
