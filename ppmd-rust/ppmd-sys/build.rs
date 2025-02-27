extern crate cc;

use std::error::Error;
use std::path::PathBuf;
use std::{env, fs, process};

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut compiler = cc::Build::new();
    compiler
        .file("libppmd/CpuArch.c")
        .file("libppmd/Ppmd7.c")
        .file("libppmd/Ppmd7aDec.c")
        .file("libppmd/Ppmd7Dec.c")
        .file("libppmd/Ppmd7Enc.c")
        .file("libppmd/Ppmd8.c")
        .file("libppmd/Ppmd8Dec.c")
        .file("libppmd/Ppmd8Enc.c")
        .opt_level(3);
    compiler.compile("libppmd.a");

    let src = env::current_dir()?.join("libppmd");
    let dst = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR environment variable")?);
    let include = dst.join("include");
    fs::create_dir_all(&include)
        .map_err(|err| format!("creating directory {}: {}", include.display(), err))?;
    for e in fs::read_dir(&src)? {
        let e = e?;
        let utf8_file_name = e
            .file_name()
            .into_string()
            .map_err(|_| format!("unable to convert file name {:?} to UTF-8", e.file_name()))?;
        if utf8_file_name.ends_with(".h") {
            let from = e.path();
            let to = include.join(e.file_name());
            fs::copy(&from, &to).map_err(|err| {
                format!("copying {} to {}: {}", from.display(), to.display(), err)
            })?;
        }
    }
    println!("cargo:root={}", dst.display());

    // Uncomment if you need to re-generate the bindings.
    //
    // let bindings = bindgen::Builder::default()
    //     .header("libppmd/Ppmd7.h")
    //     .header("libppmd/Ppmd8.h")
    //     .allowlist_function("Ppmd7_.*")
    //     .allowlist_function("Ppmd7a_.*")
    //     .allowlist_function("Ppmd7z_.*")
    //     .allowlist_function("Ppmd8_.*")
    //     .allowlist_item("PPMD7_.*")
    //     .allowlist_item("PPMD8_.*")
    //     .allowlist_recursively(true)
    //     .generate()
    //     .expect("Unable to generate bindings");
    //
    // let out_path = env::current_dir()?.join("src/bindings.rs");
    // bindings
    //     .write_to_file(out_path)
    //     .expect("Couldn't write bindings!");

    Ok(())
}
