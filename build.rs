use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run_cmake_command(args: &[&str], current_dir: &str, stage: &str) {
    let status = Command::new("cmake")
        .args(args)
        .current_dir(current_dir)
        .status()
        .unwrap_or_else(|_| panic!("Failed to execute cmake for {stage}"));
    assert!(status.success(), "Draco {stage} failed");
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn draco_cache_dir() -> &'static str {
    "third_party/install"
}

fn draco_cache_lib(target: &str) -> PathBuf {
    let cache = draco_cache_dir();
    if target.contains("windows-msvc") {
        Path::new(cache).join("lib/draco.lib")
    } else {
        Path::new(cache).join("lib/libdraco.a")
    }
}

fn draco_cache_valid(target: &str) -> bool {
    let cache = draco_cache_dir();
    Path::new(cache).join("include/draco").exists() && draco_cache_lib(target).exists()
}

fn copy_to_cache(source_install: &str, target: &str) {
    let cache = draco_cache_dir();
    if Path::new(cache).exists() {
        fs::remove_dir_all(cache).expect("Failed to remove stale Draco cache");
    }
    copy_dir_all(source_install, cache).expect("Failed to copy Draco install to cache");

    // Normalize Windows cache layout: if draco.lib ended up in the cache root,
    // move it into lib/draco.lib so the cache layout matches other platforms.
    if target.contains("windows-msvc") {
        let root_lib = Path::new(cache).join("draco.lib");
        let lib_dir = Path::new(cache).join("lib");
        if root_lib.exists() && !lib_dir.join("draco.lib").exists() {
            fs::create_dir_all(&lib_dir).unwrap();
            fs::rename(&root_lib, lib_dir.join("draco.lib")).unwrap();
        }
    }
}

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping native build on docs.rs");
        return;
    }

    if std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        println!("cargo:warning=Skipping build.rs on wasm32 target");
        return;
    }

    let target = std::env::var("TARGET").unwrap();

    // Step 1: Determine Draco install directory (cache or fresh build)
    let draco_install = if draco_cache_valid(&target) {
        println!(
            "cargo:warning=Using cached Draco from {}",
            draco_cache_dir()
        );
        draco_cache_dir().to_string()
    } else {
        println!("cargo:warning=Draco cache not found, building from source");

        let draco_build = "third_party/draco/build".to_string();
        let draco_install = format!("{draco_build}/install");

        if !Path::new(&draco_build).exists() {
            fs::create_dir_all(&draco_build).unwrap();
        }

        let status = Command::new("cmake")
            .args([
                "..",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DCMAKE_BUILD_TYPE=Release",
                "-DDRACO_TESTS=OFF",
                "-DCMAKE_INSTALL_PREFIX=install",
            ])
            .current_dir(&draco_build)
            .status()
            .expect("Failed to run CMake");
        assert!(status.success(), "CMake configuration failed");

        let (build_args, install_args) = if target.contains("windows-msvc") {
            (
                vec!["--build", ".", "--config", "Release"],
                vec!["--install", ".", "--config", "Release"],
            )
        } else {
            (vec!["--build", "."], vec!["--install", "."])
        };

        run_cmake_command(&build_args, &draco_build, "build");
        run_cmake_command(&install_args, &draco_build, "install");

        copy_to_cache(&draco_install, &target);
        draco_cache_dir().to_string()
    };

    // Step 2: Build cxx bridge
    let mut build = cxx_build::bridge("src/ffi.rs");
    build
        .file("cpp/decoder_api.cc")
        .include("include")
        .include("third_party/draco/src")
        .include("third_party/draco/build")
        .include(format!("{draco_install}/include"))
        .flag_if_supported("-std=c++17");

    if target.contains("apple-darwin") {
        build.flag("-mmacosx-version-min=15.5");
    }

    build.compile("decoder_api");

    // Step 3: Link Draco
    println!("cargo:rustc-link-search=native={draco_install}/lib");
    println!("cargo:rustc-link-lib=static=draco");

    println!("cargo:rerun-if-changed=cpp/decoder_api.cc");
    println!("cargo:rerun-if-changed=include/decoder_api.h");
    println!("cargo:rerun-if-changed=src/ffi.rs");
}
