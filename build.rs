use std::env;

use xmake::{Config, Source};

fn main() {
    // Disable building for doc.rs
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=xmake.lua");

    let mut xmake = Config::new(".");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86" || target_arch == "x86_64" {
        // match get_simd() {
        //     SimdLevel::AVX2 => {
        //         xmake.option("simd", "avx2");
        //     }
        //     SimdLevel::AVX => {
        //         xmake.option("simd", "avx");
        //     }
        //     SimdLevel::SSE4 => {
        //         xmake.option("simd", "sse4");
        //     }
        //     SimdLevel::SSE2 => {
        //         xmake.option("simd", "sse2");
        //     }
        //     _ => {}
        // }
        xmake.option("simd", "avx");
    }

    xmake.build();

    let includedirs = xmake.build_info().includedirs(Source::Package, "dlib");
    let mut cpp = cpp_build::Config::new();
    for path in includedirs {
        cpp.include(path);
    }

    cpp.flag("-std=c++14").build("src/lib.rs");
}

fn get_supported_target_features() -> std::collections::HashSet<String> {
    env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap()
        .split(',')
        .map(ToString::to_string)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimdLevel {
    None,
    SSE2,
    SSE4,
    AVX,
    AVX2,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn detect_host() -> SimdLevel {
    if std::is_x86_feature_detected!("avx2") {
        return SimdLevel::AVX2;
    } else if std::is_x86_feature_detected!("avx") {
        return SimdLevel::AVX;
    } else if std::is_x86_feature_detected!("sse4.1") || std::is_x86_feature_detected!("sse4.2") {
        return SimdLevel::SSE4;
    } else if std::is_x86_feature_detected!("sse2") {
        return SimdLevel::SSE2;
    }
    SimdLevel::None
}

pub fn detect_target() -> SimdLevel {
    let features = get_supported_target_features();

    if features.contains("avx2") {
        return SimdLevel::AVX2;
    } else if features.contains("avx") {
        return SimdLevel::AVX;
    } else if features.contains("sse4.1") || features.contains("sse4.2") {
        return SimdLevel::SSE4;
    } else if features.contains("sse2") {
        return SimdLevel::SSE2;
    }
    SimdLevel::None
}

pub fn get_simd() -> SimdLevel {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if std::env::var("HOST") == std::env::var("TARGET") {
        return detect_host();
    }

    detect_target()
}
