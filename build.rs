use std::env;

use xmake::{Config, Source};

mod embed {
    use std::env;
    use std::path::{Path, PathBuf};

    fn download_file_if_necessary_and_unzip<P: AsRef<Path>>(url: &str, path: P) {
        use bzip2::read::*;
        let path = path.as_ref();
        if path.exists() {
            return;
        }

        println!("url {}", url);

        let response = reqwest::blocking::get(url).unwrap();
        let mut decoded = BzDecoder::new(response);
        let mut file = std::fs::File::create(&path).unwrap();
        std::io::copy(&mut decoded, &mut file).unwrap();
    }

    pub fn download(file: &str, url: &str) {
        let cargo_out_dir = env::var("OUT_DIR").expect("OUT_DIR env is not set");
        let path = PathBuf::from(cargo_out_dir);

        download_file_if_necessary_and_unzip(url, path.join(file));
    }
}

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

    if cfg!(feature = "embed-fd-nn") {
        embed::download(
            "face_detector.dat",
            "https://dlib.net/files/mmod_human_face_detector.dat.bz2",
        );
    }

    if cfg!(feature = "embed-lp") {
        embed::download(
            "face_landmarks.dat",
            "https://dlib.net/files/shape_predictor_68_face_landmarks.dat.bz2",
        );
    }

    if cfg!(feature = "embed-fe-nn") {
        embed::download(
            "face_recognition.dat",
            "https://dlib.net/files/dlib_face_recognition_resnet_model_v1.dat.bz2",
        );
    }

    println!("cargo:rerun-if-changed=build.rs");

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_FD_NN");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_FE_NN");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EMBED_LP");
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
