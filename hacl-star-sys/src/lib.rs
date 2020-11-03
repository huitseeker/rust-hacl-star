#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(not(all(
    target_arch = "wasm32",
    not(any(target_os = "emscripten", target_os = "wasi"))
)))]
use libc;

#[cfg(all(
    target_arch = "wasm32",
    not(any(target_os = "emscripten", target_os = "wasi"))
))]
mod libc {
    pub type c_void = u8;
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_ulong = u32;
    pub type c_uchar = u8;
}

#[cfg(any(
    not(feature = "bindgen"),
    all(feature = "bindgen", feature = "overwrite")
))]
mod imp;

#[cfg(all(feature = "bindgen", not(feature = "overwrite")))]
mod imp {
    macro_rules! import {
        ( @import $name:ident ) => {
            pub mod $name {
                include!(concat!(env!("OUT_DIR"), "/", stringify!($name), ".rs"));
            }
        };
        ( $( pub mod $name:ident ; )* ) => {
            $(
                import!(@import $name);
            )*
        }
    }

    import! {
        pub mod hash;
        pub mod ed25519;
        pub mod curve25519;
        pub mod nacl;
    }
}

pub use imp::*;
