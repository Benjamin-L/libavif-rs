use std::env;
#[cfg(feature = "codec-rav1e")]
use std::fs;
#[cfg(feature = "codec-dav1d")]
use std::process::Command;

use cmake::Config;

fn main() {
    let mut _build_paths = String::new();
    let mut avif = Config::new("libavif");
    let out_dir = env::var("OUT_DIR").unwrap();

    #[cfg(feature = "codec-aom")]
    {
        let mut aom = Config::new("aom");
        aom.define("ENABLE_DOCS", "0")
            .define("ENABLE_EXAMPLES", "0")
            .define("ENABLE_TESTDATA", "0")
            .define("ENABLE_TESTS", "0")
            .define("ENABLE_TOOLS", "0");

        #[cfg(feature = "__internal_aom_generic_target")]
        {
            aom.define("AOM_TARGET_CPU", "generic");
        }

        aom.profile("Release").build();

        avif.define("AVIF_CODEC_AOM", "1");
        println!("cargo:rustc-link-lib=static=aom");
    }

    #[cfg(feature = "codec-rav1e")]
    {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        fs::create_dir_all(&format!("{}/include/rav1e", out_dir)).expect("mkdir");
        fs::copy(
            &format!("{}/rav1e.h", crate_dir),
            &format!("{}/include/rav1e/rav1e.h", out_dir),
        )
        .expect("copy rav1e.h");
        avif.define("AVIF_CODEC_RAV1E", "1")
            .define("AVIF_CODEC_LIBRARIES", "rav1e")
            .define("RAV1E_LIBRARY", "-rav1e");
    }

    #[cfg(feature = "codec-dav1d")]
    {
        let build_path = out_dir.clone() + "/dav1d";
        {
            println!("cargo:rustc-link-lib=static=dav1d");
            println!("cargo:rustc-link-search=native={}", build_path);
            let s = Command::new("meson")
                .arg("--default-library=static")
                .arg("--buildtype")
                .arg("release")
                .arg(&build_path)
                .arg("dav1d")
                .arg("--prefix")
                .arg(&format!("{}/install", build_path))
                .status()
                .expect("meson");
            assert!(s.success());
            let s = Command::new("ninja")
                .current_dir(&build_path)
                .arg("install")
                .status()
                .expect("ninja");
            assert!(s.success());
        }
        _build_paths += &format!("{}/install;", build_path);
        println!("cargo:rustc-link-search=native={}/src", build_path);
        avif.define("AVIF_CODEC_DAV1D", "1");
    }

    eprintln!("building libavif");

    let local_pc_files = env::join_paths(
        std::iter::once(std::path::Path::new(&out_dir).join("lib").join("pkgconfig")).chain(
            env::var("PKG_CONFIG_PATH")
                .ok()
                .iter()
                .flat_map(|v| env::split_paths(v)),
        ),
    )
    .unwrap();

    let mut avif_built = avif
        .define("CMAKE_PREFIX_PATH", &_build_paths)
        .define("BUILD_SHARED_LIBS", "0")
        .env("PKG_CONFIG_PATH", local_pc_files)
        .profile("Release")
        .build();
    avif_built.push("lib");
    println!("cargo:rustc-link-search=native={}", avif_built.display());
    println!("cargo:rustc-link-lib=static=avif");
    println!("cargo:outdir={}", out_dir);
}
