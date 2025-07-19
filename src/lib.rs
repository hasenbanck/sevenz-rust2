//! This project is a 7z compressor/decompressor written in pure Rust.
//!
//! This is a fork of the original, unmaintained sevenz-rust crate to continue the development
//! and maintenance.
//!
//! ## Supported Codecs & filters
//!
//! | Codec          | Decompression | Compression |
//! |----------------|---------------|-------------|
//! | COPY           | ✓             | ✓           |
//! | LZMA           | ✓             | ✓           |
//! | LZMA2          | ✓             | ✓           |
//! | BROTLI (*)     | ✓             | ✓           |
//! | BZIP2          | ✓             | ✓           |
//! | DEFLATE (*)    | ✓             | ✓           |
//! | PPMD           | ✓             | ✓           |
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
//! | BCJ ARM64     | ✓             |             |
//! | BCJ ARM_THUMB | ✓             |             |
//! | BCJ SPARC     | ✓             |             |
//! | DELTA         | ✓             | ✓           |
//! | BCJ2          | ✓             |             |
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;

#[cfg(feature = "compress")]
mod encoder;
/// Encoding options when compressing.
#[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
#[cfg(feature = "compress")]
pub mod encoder_options;
mod encryption;
mod error;
mod reader;

#[cfg(feature = "compress")]
mod writer;

pub(crate) mod archive;
pub(crate) mod bitset;
pub(crate) mod block;
mod codec;
pub(crate) mod decoder;
mod filter;

mod time;
#[cfg(feature = "util")]
mod util;

pub use archive::*;
pub use block::*;
pub use encryption::Password;
pub use error::Error;
pub use reader::{ArchiveReader, BlockDecoder};
pub use time::NtTime;
#[cfg(all(feature = "compress", feature = "util", not(target_arch = "wasm32")))]
pub use util::compress::*;
#[cfg(all(feature = "util", not(target_arch = "wasm32")))]
pub use util::decompress::*;
#[cfg(all(feature = "util", target_arch = "wasm32"))]
pub use util::wasm::*;
#[cfg(feature = "compress")]
pub use writer::*;
