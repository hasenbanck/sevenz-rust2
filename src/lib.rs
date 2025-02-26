//! This project is a 7z compressor/decompressor written in pure Rust.
//!
//! This is a fork of the original, unmaintained sevenz-rust crate to continue the development and maintenance.
//!
//! ## Supported Codecs & filters
//!
//! | Codec          | Decompression | Compression |
//! |----------------|---------------|-------------|
//! | COPY           | ✓             | ✓           |
//! | LZMA           | ✓             | ✓           |
//! | LZMA2          | ✓             | ✓           |
//! | BROTLI (*)     | ✓             | ✓           |
//! | BZIP2 (*)      | ✓             | ✓           |
//! | DEFLATE (*)    | ✓             | ✓           |
//! | LZ4 (*)        | ✓             | ✓           |
//! | ZSTD (*)       | ✓             | ✓           |
//!
//! (*) Require optional cargo feature.
//!
//! | Filter        | Decompression | Compression |
//! |---------------|---------------|-------------|
//! | BCJ X86       | ✓             |             |
//! | BCJ PPC       | ✓             |             |
//! | BCJ IA64      | ✓             |             |
//! | BCJ ARM       | ✓             |             |
//! | BCJ ARM_THUMB | ✓             |             |
//! | BCJ SPARC     | ✓             |             |
//! | DELTA         | ✓             |             |
//! | BCJ2          | ✓             |             |
extern crate filetime_creation as ft;
#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;
#[cfg(feature = "aes256")]
mod aes256sha256;
mod bcj;
mod bcj2;
#[cfg(feature = "brotli")]
mod brotli;
#[cfg(not(target_arch = "wasm32"))]
mod de_funcs;
mod delta;
#[cfg(feature = "compress")]
mod en_funcs;
#[cfg(feature = "compress")]
mod encoders;
mod error;
mod method_options;
mod password;
mod reader;
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(feature = "compress")]
mod writer;

#[cfg(feature = "aes256")]
pub use aes256sha256::*;
pub use archive::*;
#[cfg(not(target_arch = "wasm32"))]
pub use de_funcs::*;
#[cfg(feature = "compress")]
pub use en_funcs::*;
pub use error::Error;
pub use lzma_rust2 as lzma;
pub use method_options::*;
pub use nt_time;
pub use password::Password;
pub use reader::{BlockDecoder, SevenZReader};
#[cfg(feature = "compress")]
pub use writer::*;
pub(crate) mod archive;

pub(crate) mod decoders;
pub(crate) mod folder;
