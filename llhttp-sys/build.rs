use cc;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(feature = "gen_binding")]
extern crate bindgen;

use anyhow::{Context, Result};

fn build_llhttp(out_dir: &Path) -> Result<PathBuf> {
    generate_source(out_dir)?;
    cc::Build::new()
        .include(out_dir)
        .file(out_dir.join("llhttp.c"))
        .file(out_dir.join("api.c"))
        .file(out_dir.join("http.c"))
        .compile("llhttp");
    println!(
        "cargo:rerun-if-changed={}",
        out_dir.join("llhttp.c").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        out_dir.join("api.c").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        out_dir.join("http.c").to_str().unwrap()
    );
    Ok(PathBuf::from_str("external/llhttp/build").unwrap())
}

#[cfg(feature = "gen_source")]
fn generate_source(out_dir: &Path) -> Result<()> {
    std::process::Command::new("git")
        .current_dir(".")
        .args(["submodule", "update"])
        .output()?;
    std::process::Command::new("npm")
        .current_dir("external/llhttp")
        .args(["i"])
        .output()?;
    std::process::Command::new("npm")
        .current_dir("external/llhttp")
        .args(["run", "build"])
        .output()?;

    std::fs::copy("external/llhttp/src/native/api.c", out_dir.join("api.c"))?;
    std::fs::copy("external/llhttp/src/native/http.c", out_dir.join("http.c"))?;
    std::fs::copy("external/llhttp/build/c/llhttp.c", out_dir.join("llhttp.c"))?;
    std::fs::copy("external/llhttp/build/llhttp.h", out_dir.join("llhttp.h"))?;
    Ok(())
}

#[cfg(not(feature = "gen_source"))]
fn generate_source(out_dir: &Path) -> Result<()> {
    std::fs::copy("src/api.c", out_dir.join("api.c"))?;
    std::fs::copy("src/http.c", out_dir.join("http.c"))?;
    std::fs::copy("src/llhttp.c", out_dir.join("llhttp.c"))?;
    std::fs::copy("src/llhttp.h", out_dir.join("llhttp.h"))?;
    Ok(())
}

#[cfg(feature = "gen_binding")]
fn generate_binding(inc_dir: &Path, out_dir: &Path) -> Result<()> {
    use anyhow::Error;
    let out_file = out_dir.join("llhttp.rs");
    let inc_file = inc_dir.join("llhttp.h");
    let inc_file = inc_file.to_str().expect("header file");

    println!(
        "cargo:warning=generating raw llhttp binding file @ {} from {}",
        out_file.display(),
        inc_file,
    );

    println!("cargo:rerun-if-changed={}", inc_file);

    let llhttp_bindings = bindgen::Builder::default().header(inc_file);

    #[cfg(target_os = "macos")]
    let llhttp_bindings = llhttp_bindings
        .blocklist_type("^__darwin_.*")
        .blocklist_type("^_opaque_.*");

    llhttp_bindings
        .use_core()
        .ctypes_prefix("::libc")
        .allowlist_var("^llhttp_.*")
        .allowlist_type("^llhttp_.*")
        .allowlist_function("^llhttp_.*")
        .size_t_is_usize(true)
        .rust_target(bindgen::LATEST_STABLE_RUST)
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .newtype_enum("llhttp_errno")
        .newtype_enum("llhttp_flags")
        .newtype_enum("llhttp_lenient_flags")
        .newtype_enum("llhttp_type")
        .newtype_enum("llhttp_method")
        .generate()
        .map_err(|_| Error::msg("generate binding files"))?
        .write_to_file(out_file)
        .with_context(|| "write wrapper")?;

    Ok(())
}

#[cfg(not(feature = "gen_binding"))]
fn generate_binding(_: &Path, out_dir: &Path) -> Result<()> {
    std::fs::copy("src/llhttp.rs", out_dir.join("llhttp.rs"))
        .map(|_| ())
        .with_context(|| "copy binding file")
}

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);
    let inc_dir = build_llhttp(out_dir)?;

    generate_binding(&inc_dir, &out_dir)?;

    Ok(())
}
