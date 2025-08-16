use std::{env, path::PathBuf};
use std::process::Command;

fn main() {
    // Where the ggml sources live relative to this crate
    let ggml_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("ggml");

    // Build ggml via CMake with static libraries. GPU backends are feature-gated.
    let mut cfg = cmake::Config::new(&ggml_src);
    cfg.profile("Release");
    cfg.define("BUILD_SHARED_LIBS", "OFF")
        .define("GGML_BUILD_TESTS", "OFF")
        .define("GGML_BUILD_EXAMPLES", "OFF")
        .define("GGML_BACKEND_DL", "OFF")
        // CPU-only libs by default; conditionally enable GPU backends below
        .define("GGML_ACCELERATE", "OFF")
        .define("GGML_BLAS", "OFF");

    // Cargo sets env vars CARGO_FEATURE_<FEATURE_NAME_IN_CAPS> for enabled features
    let has_feature = |name: &str| env::var(name).is_ok();
    let feat_cuda = has_feature("CARGO_FEATURE_CUDA");
    let feat_hip = has_feature("CARGO_FEATURE_HIP");
    let feat_musa = has_feature("CARGO_FEATURE_MUSA");
    let feat_vulkan = has_feature("CARGO_FEATURE_VULKAN");
    let feat_webgpu = has_feature("CARGO_FEATURE_WEBGPU");
    let feat_metal = has_feature("CARGO_FEATURE_METAL");
    let feat_opencl = has_feature("CARGO_FEATURE_OPENCL");
    let feat_sycl = has_feature("CARGO_FEATURE_SYCL");

    cfg.define("GGML_CUDA", if feat_cuda { "ON" } else { "OFF" })
        .define("GGML_HIP", if feat_hip { "ON" } else { "OFF" })
        .define("GGML_MUSA", if feat_musa { "ON" } else { "OFF" })
        .define("GGML_VULKAN", if feat_vulkan { "ON" } else { "OFF" })
        .define("GGML_WEBGPU", if feat_webgpu { "ON" } else { "OFF" })
        .define("GGML_METAL", if feat_metal { "ON" } else { "OFF" })
        .define("GGML_OPENCL", if feat_opencl { "ON" } else { "OFF" })
        .define("GGML_SYCL", if feat_sycl { "ON" } else { "OFF" });

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

    // Conditionally link GPU backend libraries and required system deps
    if feat_metal {
        println!("cargo:rustc-link-lib=static=ggml-metal");
        // On Apple platforms, link required frameworks
        match target_os.as_str() {
            "macos" | "ios" | "tvos" | "watchos" | "visionos" => {
                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=MetalKit");
                println!("cargo:rustc-link-lib=framework=Foundation");
            }
            _ => {}
        }
    }
    if feat_cuda {
        println!("cargo:rustc-link-lib=static=ggml-cuda");
        // CUDA runtime and BLAS; names vary per platform
        match target_os.as_str() {
            "windows" => {
                println!("cargo:rustc-link-lib=cudart");
                println!("cargo:rustc-link-lib=cublas");
                // CUDA driver on Windows is nvcuda
                println!("cargo:rustc-link-lib=nvcuda");
            }
            _ => {
                println!("cargo:rustc-link-lib=cudart");
                println!("cargo:rustc-link-lib=cublas");
                // CUDA driver on Unix-like systems
                println!("cargo:rustc-link-lib=cuda");
            }
        }
    }
    if feat_vulkan {
        println!("cargo:rustc-link-lib=static=ggml-vulkan");
        // System Vulkan loader
        match target_os.as_str() {
            "windows" => println!("cargo:rustc-link-lib=vulkan-1"),
            _ => println!("cargo:rustc-link-lib=vulkan"),
        }
    }
    if feat_opencl {
        println!("cargo:rustc-link-lib=static=ggml-opencl");
        match target_os.as_str() {
            "macos" | "ios" | "tvos" | "watchos" => {
                println!("cargo:rustc-link-lib=framework=OpenCL");
            }
            _ => {
                println!("cargo:rustc-link-lib=OpenCL");
            }
        }
    }
    if feat_hip {
        println!("cargo:rustc-link-lib=static=ggml-hip");
    }
    if feat_musa {
        println!("cargo:rustc-link-lib=static=ggml-musa");
    }
    if feat_sycl {
        println!("cargo:rustc-link-lib=static=ggml-sycl");
    }

    // Rebuild if headers change
    println!(
        "cargo:rerun-if-changed={}",
        ggml_src.join("include").display()
    );
    println!("cargo:rerun-if-changed=wrapper.h");

    // Expose git/ build metadata to Rust code and docs
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ggml_dir = manifest_dir.join("ggml");
    let git_out = |dir: &PathBuf, args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .ok()?;
        if out.status.success() {
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            None
        }
    };
    // ggml submodule commit hash and commit time
    let ggml_commit = git_out(&ggml_dir, &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let ggml_time = git_out(&ggml_dir, &["show", "-s", "--format=%cI", "HEAD"]).unwrap_or_else(|| {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => format!("{}", d.as_secs()),
            Err(_) => "unknown".into(),
        }
    });
    println!("cargo:rustc-env=GGML_FFI_GGML_COMMIT={}", ggml_commit);
    println!("cargo:rustc-env=GGML_FFI_GGML_COMMIT_TIME={}", ggml_time);

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
        // Make C enums into proper Rust enums where safe
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: true })
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
