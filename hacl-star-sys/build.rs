#[cfg(feature = "bindgen")]
extern crate bindgen;
extern crate cc;

use std::env;
#[cfg(feature = "bindgen")]
use std::path::PathBuf;

fn main() {
    let mut cc = cc::Build::new();

    // from https://github.com/project-everest/hacl-star/blob/master/snapshots/makefiles/CMakeLists.txt#L62
    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH") == Ok("32".into())
        || env::var("CARGO_CFG_TARGET_ENV") == Ok("msvc".into())
    {
        cc.shared_flag(true)
            .define("KRML_NOUINT128", None)
            .flag_if_supported("-Wno-unused-function")
            .file("hacl-c/portable-gcc-compatible/FStar.c");
    }

    cc.flag_if_supported(
        if cc::Build::new().get_compiler().is_like_gnu()
            || cc::Build::new().get_compiler().is_like_clang()
        {
            "-std=gnu11"
        } else {
            "-std=c11"
        },
    )
    .include("hacl-c/portable-gcc-compatible")
    // .include("hacl-c/kremlin")
    .include("hacl-c/kremlin/include")
    .include("hacl-c/kremlin/kremlib/dist/minimal")
    // from https://github.com/mitls/hacl-star/blob/master/snapshots/hacl-c/portable-gcc-compatible/Makefile#L8
    .flag_if_supported("-fwrapv")
    .flag_if_supported("-fomit-frame-pointer")
    .flag_if_supported("-funroll-loops")
    .files(&[
        "hacl-c/portable-gcc-compatible/Hacl_Hash.c",
        // "hacl-c/portable-gcc-compatible/Hacl_SHA2_256.c",
        // "hacl-c/portable-gcc-compatible/Hacl_SHA2_384.c",
        // "hacl-c/portable-gcc-compatible/Hacl_SHA2_512.c",
        "hacl-c/portable-gcc-compatible/Hacl_Ed25519.c",
        // "hacl-c/portable-gcc-compatible/Hacl_Curve25519_64.c",
        "hacl-c/portable-gcc-compatible/Hacl_Curve25519_51.c",
        // "hacl-c/portable-gcc-compatible/Hacl_Policies.c",
        "hacl-c/portable-gcc-compatible/Hacl_NaCl.c",
    ])
    // ignore some warnings
    .flag_if_supported("-Wno-unused-function")
    .flag_if_supported("-Wno-unused-parameter")
    .flag_if_supported("-Wno-unused-variable")
    .compile("hacl");

    #[cfg(all(feature = "bindgen", feature = "overwrite"))]
    let outdir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("imp");

    #[cfg(all(feature = "bindgen", not(feature = "overwrite")))]
    let outdir = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "bindgen")]
    macro_rules! bindgen {
        ( @bind $input:expr => $output:expr , $white:expr ) => {
            bindgen::Builder::default()
                .header($input)
                .ctypes_prefix("crate::libc")
                .use_core()
                .whitelist_type($white)
                .whitelist_function($white)
                .whitelist_var($white)
                .clang_arg("-I//home/huitseeker/tmp/rust-hacl-star/hacl-star-sys/hacl-c/kremlin/include/")
                .clang_arg("-I//home/huitseeker/tmp/rust-hacl-star/hacl-star-sys/hacl-c/kremlin/kremlib/dist/minimal")
                .generate().unwrap()
                .write_to_file(outdir.join($output)).unwrap();
        };
        ( $( $input:expr => $output:expr , $white:expr );* ) => {
            $(
                bindgen!(@bind $input => $output, $white);
            )*
        }
    }

    #[cfg(feature = "bindgen")]
    bindgen! {
        "hacl-c/portable-gcc-compatible/Hacl_Hash.h"         => "hash.rs",           "Hacl_Hash_.+";
        // "hacl-c/portable-gcc-compatible/Hacl_SHA2_256.h"         => "sha2_256.rs",           "Hacl_SHA2_256_.+";
        // "hacl-c/portable-gcc-compatible/Hacl_SHA2_384.h"         => "sha2_384.rs",           "Hacl_SHA2_384_.+";
        // "hacl-c/portable-gcc-compatible/Hacl_SHA2_512.h"         => "sha2_512.rs",           "Hacl_SHA2_512_.+";
        "hacl-c/portable-gcc-compatible/Hacl_Ed25519.h"          => "ed25519.rs",            "Hacl_Ed25519_.+";
        // "hacl-c/portable-gcc-compatible/Hacl_Curve25519_64.h"       => "curve25519_64.rs",         "Hacl_Curve25519_64_.+";
        "hacl-c/portable-gcc-compatible/Hacl_Curve25519_51.h"       => "curve25519.rs",         "Hacl_Curve25519_.+";
        // "hacl-c/portable-gcc-compatible/Hacl_Policies.h"         => "hacl_policies.rs",      "Hacl_Policies_.+";
        "hacl-c/portable-gcc-compatible/Hacl_NaCl.h"                  => "nacl.rs",               "NaCl_.+"
    };
}
