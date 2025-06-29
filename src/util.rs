#[cfg(all(feature = "compress", feature = "util"))]
pub(crate) mod compress;
#[cfg(all(feature = "util", not(target_arch = "wasm32")))]
pub(crate) mod decompress;

#[cfg(target_arch = "wasm32")]
mod wasm;
