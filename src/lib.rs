mod core;

#[cfg(not(target_arch = "wasm32"))]
pub mod napi;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
// Force rebuild

