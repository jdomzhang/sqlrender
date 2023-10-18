#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(not(target_arch = "wasm32"))]
include!("lib_inner.rs");

#[cfg(target_arch = "wasm32")]
pub use sqlrender_impl::SqlRender;

// #[cfg(target_arch = "wasm32")]
// pub fn now_ms() -> i64 {
// 	panic!("now_ms() is not implemented for wasm32");
// }
