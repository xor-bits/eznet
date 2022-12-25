use std::{fs::File, io::Write, path::Path};

//

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let pkg_vers_maj = std::env::var_os("CARGO_PKG_VERSION_MAJOR")
        .unwrap()
        .into_string()
        .unwrap();
    let pkg_vers_min = std::env::var_os("CARGO_PKG_VERSION_MINOR")
        .unwrap()
        .into_string()
        .unwrap();
    let dest_path = Path::new(&out_dir).join("version");
    let mut file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dest_path)
        .unwrap();

    writeln!(file, "({pkg_vers_maj}, {pkg_vers_min})",).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
