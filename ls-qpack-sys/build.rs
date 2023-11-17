// Copyright 2022 Biagio Festa

use std::env;
use std::path::PathBuf;

fn get_abi(target: String) -> String {
    if target.contains("armv7") {
        "armeabi-v7a".to_string()
    } else if target.contains("aarch64") {
        "arm64-v8a".to_string()
    } else if target.contains("i686") {
        "x86".to_string()
    } else if target.contains("x86_64") {
        "x86_64".to_string()
    } else {
        panic!("Unknown target {}", target);
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ls_qpack_dep_dir = PathBuf::from("deps/ls-qpack");

    let target = env::var("TARGET").unwrap();

    if target.contains("android") {
        let ndk_home = env::var("ANDROID_NDK_HOME").unwrap();
        let abi = get_abi(target.clone());
        cmake::Config::new(&ls_qpack_dep_dir)
            .define(
                "CMAKE_TOOLCHAIN_FILE",
                format!("{}/build/cmake/android.toolchain.cmake", ndk_home),
            )
            .define("LSQPACK_BIN", "OFF")
            .define("ANDROID_ABI", abi)
            .build();
    } else {
        cmake::Config::new(&ls_qpack_dep_dir)
            .define("LSQPACK_BIN", "OFF")
            .build();
    }

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=ls-qpack");

    let builder;

    if target.contains("android") {
        let include_target;
        if target.contains("armv7") {
            include_target = "arm-linux-androideabi".to_string();
        } else {
            include_target = target;
        }
        let ndk_home = env::var("ANDROID_NDK_HOME").unwrap();
        builder = bindgen::Builder::default()
            .derive_debug(true)
            .derive_default(true)
            .derive_copy(false)
            .size_t_is_usize(true)
            .layout_tests(true)
            .generate_comments(false)
            .clang_arg(format!("-I{}", out_dir.join("include").display()))
            .clang_arg(format!(
                "-I{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include",
                ndk_home
            ))
            .clang_arg(format!(
                "-I{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include/{}",
                ndk_home, include_target
            ))
            .header(
                ls_qpack_dep_dir
                    .join("lsqpack.h")
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            )
            .header(
                ls_qpack_dep_dir
                    .join("lsxpack_header.h")
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            );
    } else {
        builder = bindgen::Builder::default()
            .derive_debug(true)
            .derive_default(true)
            .derive_copy(false)
            .size_t_is_usize(true)
            .layout_tests(true)
            .generate_comments(false)
            .clang_arg(format!("-I{}", out_dir.join("include").display()))
            .header(
                ls_qpack_dep_dir
                    .join("lsqpack.h")
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            )
            .header(
                ls_qpack_dep_dir
                    .join("lsxpack_header.h")
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            );
    }

    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
