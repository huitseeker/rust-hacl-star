use std::{ fs, env };
use std::borrow::Cow;


const CODE_TARGET: &str = "c89-compatible";
const CFLAGS: &str = "-Wall -Wextra -Werror -std=c11 -Wno-unused-variable \
    -Wno-unknown-warning-option -Wno-unused-but-set-variable \
    -Wno-unused-parameter -Wno-infinite-recursion \
    -g -fwrapv -D_BSD_SOURCE -D_DEFAULT_SOURCE";
const CFLAGS2: &str = "-Wno-parentheses -Wno-deprecated-declarations -g -std=gnu11 -O3";


fn parse_include(input: &str) -> Option<(&str, &str, &str, &str, &str)> {
    let mut lines = input.lines().take(5);

    macro_rules! parse {
        ( $input:expr, $key:expr ) => {{
            let mut input = $input.next()?.splitn(2, '=');
            assert_eq!(input.next()?, $key);
            input.next()?
        }}
    }

    Some((
        parse!(lines, "USER_TARGET"),
        parse!(lines, "USER_CFLAGS"),
        parse!(lines, "USER_C_FILES"),
        parse!(lines, "ALL_C_FILES"),
        parse!(lines, "ALL_H_FILES"),
    ))
}

fn make_evercrypt(
    code_target: &str,
    user_target: &str,
    user_cflags: &str,
    user_c_files: &str,
    all_c_files: &str
) {
    let mut builder = cc::Build::new();

    // include kremlin
    builder
        .include("./hacl-star/dist/kremlin/include")
        .include("./hacl-star/dist/kremlin/kremlib/dist/minimal")
        .include(format!("./hacl-star/dist/{}", code_target));


    for flag in CFLAGS.split(' ').chain(user_cflags.split(' ')).chain(CFLAGS2.split(' ')) {
        builder.flag_if_supported(flag.trim());
    }

    for file in all_c_files.split(' ').chain(user_c_files.split(' ')) {
        builder.file(format!("./hacl-star/dist/{}/{}", code_target, file.trim()));
    }

    match env::var("CARGO_CFG_TARGET_FAMILY").as_ref().map(String::as_str) {
        Ok("unix") => {
            builder
                .flag_if_supported("-fPIC")
                .flag_if_supported("-fstack-check");
        },
        Ok("windows") => {
            builder
                .flag("-D__USE_MINGW_ANSI_STDIO")
                .flag("-fno-asynchronous-unwind-tables");
        },
        _ => ()
    }

    builder.compile(user_target.trim_start_matches("lib").trim_end_matches(".a"));
}

#[cfg(feature = "bindgen")]
fn make_binding(code_target: &str, all_h_files: &str) {
    use std::path::Path;

    #[cfg(feature = "overwrite")]
    let outdir = Path::new(env::var("CARGO_MANIFEST_DIR").as_ref().unwrap())
        .join("src");

    #[cfg(not(feature = "overwrite"))]
    let outdir = Path::new(env::var("OUT_DIR").as_ref().unwrap())
        .to_owned();

    let mut builder = bindgen::Builder::default();

    if env::var("CARGO_CFG_TARGET_ARCH") == Ok("wasm".into()) {
        builder = builder.clang_arg("-fvisibility=default");
    }

    builder = builder
        .clang_arg("-I./hacl-star/dist/kremlin/include")
        .clang_arg("-I./hacl-star/dist/kremlin/kremlib/dist/minimal")
        .clang_arg(format!("-I./hacl-star/dist/{}", code_target));

    for file in all_h_files.split(' ') {
        builder = builder.header(format!("./hacl-star/dist/{}/{}", code_target, file.trim()));
    }

    builder
        .ctypes_prefix("crate::libc")
        .whitelist_function("EverCrypt.*|Spec.*")
        .whitelist_type("EverCrypt.*|Spec.*")
        .whitelist_var("EverCrypt.*|Spec.*")
        .use_core()
        .generate().unwrap()
        .write_to_file(outdir.join("sys.rs")).unwrap();
}

fn main() {
    let code_target = env::var("HACL_CODE_TARGET")
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(CODE_TARGET));

    let include =
        fs::read_to_string(format!("./hacl-star/dist/{}/Makefile.include", code_target)).unwrap();

    #[allow(unused_variables)]
    let (user_target, user_cflags, user_c_files, all_c_files, all_h_files) =
        parse_include(&include).expect("makefile.include parse failed");

    make_evercrypt(&code_target, user_target, user_cflags, user_c_files, all_c_files);

    #[cfg(feature = "bindgen")]
    make_binding(&code_target, all_h_files);
}
