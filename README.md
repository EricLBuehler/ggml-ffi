ggml-ffi
=========

Low-level Rust FFI bindings to the GGML C API shipped in this repository under `ggml/`.

Features
--------

- metal: Enable GGML_METAL backend and link Metal frameworks on Apple.
- cuda: Enable GGML_CUDA backend and link CUDA runtime/BLAS libraries.
- vulkan: Enable GGML_VULKAN backend and link libvulkan.
- opencl: Enable GGML_OPENCL backend and link OpenCL (framework on Apple).
- hip: Enable GGML_HIP backend (requires ROCm toolchain).
- musa: Enable GGML_MUSA backend.
- sycl: Enable GGML_SYCL backend (requires SYCL toolchain).
- webgpu: Enable GGML_WEBGPU backend (requires WebGPU support in ggml).

Examples
--------

- CPU only (default): `cargo build`
- Metal on macOS: `cargo build --features metal`
- CUDA on Linux: `cargo build --features cuda`
- Vulkan: `cargo build --features vulkan`
- OpenCL: `cargo build --features opencl`

Note: Enabling a backend requires the corresponding SDK/toolchain to be installed and discoverable by CMake.
