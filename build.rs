use std::{env, path::PathBuf};

fn main() {
    // Where the ggml sources live relative to this crate
    let ggml_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("ggml");

    // Build ggml via CMake with CPU-only static libraries to simplify linking
    let mut cfg = cmake::Config::new(&ggml_src);
    cfg.profile("Release");
    cfg.define("BUILD_SHARED_LIBS", "OFF")
        .define("GGML_BUILD_TESTS", "OFF")
        .define("GGML_BUILD_EXAMPLES", "OFF")
        .define("GGML_BACKEND_DL", "OFF")
        .define("GGML_ACCELERATE", "OFF")
        .define("GGML_BLAS", "OFF")
        .define("GGML_CUDA", "OFF")
        .define("GGML_HIP", "OFF")
        .define("GGML_MUSA", "OFF")
        .define("GGML_VULKAN", "OFF")
        .define("GGML_WEBGPU", "OFF")
        .define("GGML_METAL", "OFF")
        .define("GGML_OPENCL", "OFF");

    let dst = cfg.build();

    // Tell cargo to link the built static libraries
    let lib_dir = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    // Core libs
    println!("cargo:rustc-link-lib=static=ggml-base");
    println!("cargo:rustc-link-lib=static=ggml");
    // CPU backend
    println!("cargo:rustc-link-lib=static=ggml-cpu");

    // Also link necessary system libs depending on target
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        // Linux and similar often need libstdc++, libm, libpthread, and libdl
        "linux" | "freebsd" | "netbsd" | "openbsd" => {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=dl");
        }
        // macOS family uses libc++
        "macos" | "ios" | "tvos" | "watchos" => {
            println!("cargo:rustc-link-lib=c++");
        }
        // Android typically needs c++_shared when using NDK; keep minimal here
        "android" => {
            println!("cargo:rustc-link-lib=c++_shared");
            println!("cargo:rustc-link-lib=dl");
        }
        _ => {}
    }

    // Rebuild if headers change
    println!(
        "cargo:rerun-if-changed={}",
        ggml_src.join("include").display()
    );
    println!("cargo:rerun-if-changed=wrapper.h");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Make sure clang sees ggml's public headers
        .clang_arg(format!("-I{}", ggml_src.join("include").display()))
        // Keep the namespace small: only ggml/gguf APIs
        .allowlist_function("ggml_.*")
        .allowlist_function("gguf_.*")
        .allowlist_type("ggml_.*")
        .allowlist_type("gguf_.*")
        .allowlist_var("GGML_.*")
        .allowlist_var("GGUF_.*")
        // Derives to make the bindings ergonomic
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .layout_tests(false)
        .generate()
        .expect("unable to generate ggml bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("could not write bindings");
}
