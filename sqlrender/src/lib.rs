#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(not(target_arch = "wasm32"))]
include!("lib_inner.rs");

#[cfg(target_arch = "wasm32")]
pub use sqlrender_impl::SqlRender;
